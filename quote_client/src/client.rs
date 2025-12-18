use std::fmt::format;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::thread;
use std::time::{Duration};
use socket2::{Domain, Protocol, Socket, Type};
use quote_lib::quote::stockquote::StockQuote;
use crate::error::QuoteClientError;

#[derive(Default)]
pub(crate) struct QuoteStreamClient {
    is_running_ping: Arc<AtomicBool>,
    remote_add: Arc<Mutex<String>>
}
const UDP_READ_TIMEOUT_SECOND: u64 = 4;
const DURATION_WAIT_TO_CONNECT: u64 = 10;

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
                    .with_time(Duration::from_secs(10))
                    .with_interval(Duration::from_secs(5)),
            )?;
        }
        socket.connect(&socket_addr.clone().into())?;
        let stream: TcpStream = socket.into();
        // тайм-аут на чтение
        stream.set_read_timeout(Some(Duration::from_secs(3)))?;
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
            socket.send_to("PING".as_bytes(), &send_addr)?;
            thread::sleep(Duration::from_secs(2));
        }
        is_running_ping.store(false, SeqCst);
        Ok(())
    }

    pub fn get_quote_stream(&mut self, udp_bind_adr: &str, server_adr: &str, tickers: String) -> Result<(), QuoteClientError> {
        let socket = UdpSocket::bind(udp_bind_adr)?;
        socket.set_read_timeout(Some(Duration::from_secs(UDP_READ_TIMEOUT_SECOND)))?;
        let mut is_connected = false;
        loop {
            if !is_connected{
                while self.is_running_ping.load(SeqCst) {
                    self.is_running_ping.store(true, SeqCst);
                }
                if let Ok(stream) = QuoteStreamClient::connect(server_adr) {
                    let mut writer = stream.try_clone().expect("failed to clone stream");
                    let mut reader = BufReader::new(stream);
                    match writer.write_all(format!("STREAM udp://{} {}", udp_bind_adr, tickers).as_bytes()){
                        Ok(_) => {
                            writer.flush()?;
                            let mut result = String::new();
                            if let Ok(_) = reader.read_line(&mut result) && result.contains("OK") {
                                is_connected = true;
                            }
                        }
                        Err(_) => {
                            thread::sleep(Duration::from_secs(DURATION_WAIT_TO_CONNECT));
                            continue
                        }
                    }
                }
            }
            if !self.is_running_ping.load(SeqCst) {
                is_connected = false;
            }
            let mut quote: Vec<u8> = Vec::new();
            match socket.recv_from(&mut quote) {
                Ok((size, src)) => {
                    if size > 0 {
                        if let Some(quote) = StockQuote::from_string(String::from_utf8_lossy(&quote[..size]).as_ref()) {
                            println!("{:?}", quote);
                        }
                        let udp = socket.try_clone()?;
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
                        }
                    }
                }
                Err(e) => {
                    is_connected = false;
                }
            }
        }
    }
}