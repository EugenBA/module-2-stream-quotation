use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::thread;
use std::time::{Duration};
use socket2::{Domain, Protocol, Socket, Type};
use quote_lib::quote::stockquote::StockQuote;
use crate::error::QuoteClientError;
use log;

#[derive(Default)]
pub(crate) struct QuoteStreamClient {
    is_running_ping: Arc<AtomicBool>,
    remote_add: Arc<Mutex<String>>
}

//константа таймаут чтения udp сек
const UDP_READ_TIMEOUT_SECOND: u64 = 4;
// константа таймаут времнеи на установление подключения сек
const DURATION_WAIT_TO_CONNECT: u64 = 10;
//констата таймаут времени чтения данных
const DURATION_READ_TIMEOUT: u64 = 5;
//костаната параметра with_time tcp_keepalive
const TCP_KEEPALIVE_WITH_TIME: u64 = 10;
//константа парметра with_interval tcp_keepalive
const TCP_KEEPALIVE_WITH_INTERVAL: u64 = 5;
//константа паузы потока отправки данных PING
const PING_SEND_THREAD_WAIT: u64 = 2;

impl QuoteStreamClient {
    fn connect(server_addr: &str) -> Result<TcpStream, QuoteClientError> {
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        let socket_addr = server_addr.parse::<SocketAddr>()?;
        // Включаем TCP keepalive
        socket.set_keepalive(true)?;
       #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            socket.set_tcp_keepalive(
                &socket2::TcpKeepalive::new()
                    .with_time(Duration::from_secs(TCP_KEEPALIVE_WITH_TIME))
                    .with_interval(Duration::from_secs(TCP_KEEPALIVE_WITH_INTERVAL)),
            )?;
        }
        socket.connect(&socket_addr.clone().into())?;
        let stream: TcpStream = socket.into();
        // тайм-аут на чтение
        stream.set_read_timeout(Some(Duration::from_secs(DURATION_READ_TIMEOUT)))?;
        Ok(stream)
    }

    fn thread_ping_quote_server(socket: UdpSocket,
                                server_adr: Arc<Mutex<String>>,
                                is_running_ping: Arc<AtomicBool>) -> Result<(), QuoteClientError> {
        is_running_ping.store(true, SeqCst);
        let mut send_addr = String::new();
        loop {
            if !is_running_ping.load(SeqCst) {
                break
            }
            if let Ok(server_adr) = server_adr.lock() {
                send_addr = server_adr.to_string();
            }
            socket.send_to("PING\n".as_bytes(), &send_addr)?;
            thread::sleep(Duration::from_secs(PING_SEND_THREAD_WAIT));
        }
        is_running_ping.store(false, SeqCst);
        Ok(())
    }

    pub fn get_quote_stream(&mut self, udp_bind_adr: &str, server_adr: &str, tickers: String) -> Result<(), QuoteClientError> {
        let socket = UdpSocket::bind(udp_bind_adr)?;
        socket.set_read_timeout(Some(Duration::from_secs(UDP_READ_TIMEOUT_SECOND)))?;
        let mut is_connected = false;
        let mut udp_src_addr = String::new();
        loop {
            //подключаемся к серверу
            if !is_connected {
                log::info!("try connecting to server at {}", server_adr);
                udp_src_addr = "".to_string();
                while self.is_running_ping.load(SeqCst) {
                    self.is_running_ping.store(false, SeqCst);
                }
                match QuoteStreamClient::connect(server_adr) {
                    Ok(stream) => {
                        let mut writer = stream.try_clone().expect("failed to clone stream");
                        let mut reader = BufReader::new(stream);
                        let mut result = String::new();
                        //отправляем команду для получения данных
                        writer.write_all(format!("STREAM udp://{} {}\n", udp_bind_adr, tickers).as_bytes())?;
                        writer.flush()?;
                        loop {
                            match reader.read_line(&mut result) {
                                Ok(0) => {
                                    is_connected = false;
                                    break;
                                }
                                Ok(_) => {
                                    //сервер ответил сообщение ОК, коннект установлен
                                    if result.contains("OK") {
                                        is_connected = true;
                                        break;
                                    }
                                },
                                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                                    result.clear();
                                    log::error!("waiting for server response...");
                                    thread::sleep(Duration::from_millis(DURATION_WAIT_TO_CONNECT));
                                    continue;
                                },
                                Err(_) => {
                                    is_connected = false;
                                    break;
                                }
                            }
                        }
                    },
                    Err(_) => {
                        thread::sleep(Duration::from_secs(DURATION_WAIT_TO_CONNECT));
                        return Err(QuoteClientError::BadNetworkBindSocket(format!("Error connect address {}", server_adr)));
                    }
                }
            }
            let mut quote = [0u8; 1024];
            // читаем данные из udp сокета
            match socket.recv_from(&mut quote) {
                Ok((0,_)) => {
                    is_connected = false;
                    continue;
                }
                //данные по котировкам
                Ok((size, src)) => {
                    if size > 0 {
                         if let Some(quote) = StockQuote::from_string(String::from_utf8_lossy(&quote[..size]).as_ref())
                         {
                             println!("{:?}", quote);
                         }
                    }
                    //определяеи адрес отправителя, чтоб отправить сообщения PING
                    if src.to_string() != udp_src_addr {
                        udp_src_addr = src.to_string();
                        while self.is_running_ping.load(SeqCst) {
                            self.is_running_ping.store(false, SeqCst);
                        }
                        let udp = socket.try_clone()?;
                        // создаем отдельны поток для оправки данных PING
                        if let Ok(mut server_adr) = self.remote_add.lock() {
                            *server_adr = src.to_string();
                            let is_running_ping = self.is_running_ping.clone();
                            let server_adr = self.remote_add.clone();
                            thread::spawn(move || {
                                QuoteStreamClient::thread_ping_quote_server(
                                    udp,
                                    server_adr,
                                    is_running_ping
                                )
                            });
                            while !self.is_running_ping.load(SeqCst) {
                                log::warn!("waiting starting ping quote server...");
                                thread::sleep(Duration::from_millis(DURATION_WAIT_TO_CONNECT));
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("error read quote stream: {}", e);
                    is_connected = false;
                }
            }
            if !self.is_running_ping.load(SeqCst) {
                is_connected = false;
            }
        }
    }
}

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn test_connect() {
        //error test
        let url = "127.0.0.1:8120";
        let test_connect = QuoteStreamClient::connect(url);
        assert_eq!(test_connect.err().unwrap(), QuoteClientError::BadNetworkBindSocket("Connection refused (os error 111)".to_string()));
    }

    #[test]
    fn test_get_quote_stream(){
        //error test
        let url = "127.0.0.1:8120";
        let tickers = "MSFT,GOOG,AAPL".to_string();
        let mut test_client = QuoteStreamClient::default();
        let test_connect = test_client.get_quote_stream(url, url, tickers);
        assert_eq!(test_connect.err().unwrap(), QuoteClientError::BadNetworkBindSocket("Error connect address 127.0.0.1:8120".to_string()));
    }

}