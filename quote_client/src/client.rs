use std::fmt::format;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::thread;
use std::time::{Duration};
use clap::builder::Str;
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
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
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
            thread::sleep(Duration::from_secs(2));
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
            if !is_connected {
                println!("Try connecting to server at {}", server_adr);
                udp_src_addr = "".to_string();
                while self.is_running_ping.load(SeqCst) {
                    self.is_running_ping.store(false, SeqCst);
                }
                match QuoteStreamClient::connect(server_adr) {
                    Ok(stream) => {
                        let mut writer = stream.try_clone().expect("failed to clone stream");
                        let mut reader = BufReader::new(stream);
                        let mut result = String::new();
                        writer.write_all(format!("STREAM udp://{} {}\n", udp_bind_adr, tickers).as_bytes())?;
                        writer.flush()?;
                        loop {
                            match reader.read_line(&mut result) {
                                Ok(0) => {
                                    is_connected = false;
                                    break;
                                }
                                Ok(_) => {
                                    if result.contains("OK") {
                                        is_connected = true;
                                        break;
                                    }
                                },
                                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                                    result.clear();
                                    println!("Waiting for server response...");
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
                        continue;
                    }
                }
            }
            let mut quote = [0u8; 1024];
            match socket.recv_from(&mut quote) {
                Ok((0,_)) => {
                    is_connected = false;
                    continue;
                }
                Ok((size, src)) => {
                    if size > 0 {
                         if let Some(quote) = StockQuote::from_string(String::from_utf8_lossy(&quote[..size]).as_ref())
                         {
                             println!("{:?}", quote);
                         }
                    }
                    if src.to_string() != udp_src_addr {
                        udp_src_addr = src.to_string();
                        while self.is_running_ping.load(SeqCst) {
                            self.is_running_ping.store(false, SeqCst);
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
                            while !self.is_running_ping.load(SeqCst) {
                                println!("Waiting starting ping quote server...");
                                thread::sleep(Duration::from_millis(DURATION_WAIT_TO_CONNECT));
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Error read quote stream: {}", e);
                    is_connected = false;
                }
            }
            if !self.is_running_ping.load(SeqCst) {
                is_connected = false;
            }
        }
    }
}