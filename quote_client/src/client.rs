use std::fmt::format;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::thread;
use std::time::{Duration};
use socket2::{Domain, Protocol, Socket, Type};
use quote_lib::quote::stockquote::StockQuote;
use crate::error::QuoteClientError;

pub(crate) struct QuoteStreamClient {
    socket: UdpSocket,
    is_running_ping: Arc<AtomicBool>,
    remote_add: Arc<RwLock<String>>
}
const UDP_READ_TIMEOUT_SECOND: u64 = 4;
const DURATION_WAIT_TO_CONNECT: u64 = 10;

impl QuoteStreamClient {
    pub fn new(bind_adr: &str) -> Result<Self, QuoteClientError> {
        Ok(Self {
            socket: UdpSocket::bind(bind_adr)?,
            is_running_ping: Arc::new(AtomicBool::new(false)),
            remote_add: Arc::new(Default::default()),
        })
    }

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
                                server_adr: Arc<RwLock<String>>,
                                is_running_ping: Arc<AtomicBool>) -> Result<(), QuoteClientError>{
        is_running_ping.store(true, SeqCst);
        if let Ok(server_adr) = server_adr.read(){
            let server_adr = server_adr.clone();
            loop {
                if !is_running_ping.load(SeqCst) {
                    break
                }
                socket.send_to("PING".as_bytes(), &server_adr)?;
            }
        }
        is_running_ping.store(false, SeqCst);
        Ok(())
    }

    pub fn get_quote_stream(udp_bind_adr: &str, server_adr: &str, tickers: String) -> Result<(), QuoteClientError> {
        let socket = Self::new(udp_bind_adr)?;
        socket.socket.set_read_timeout(Some(Duration::from_secs(UDP_READ_TIMEOUT_SECOND)))?;
        let mut is_connected = false;
        loop {
            if !is_connected{
                while socket.is_running_ping.load(SeqCst) {
                    socket.is_running_ping.store(true, SeqCst);
                }
                if let Ok(stream) = QuoteStreamClient::connect(server_adr) {
                    let mut writer = stream.try_clone().expect("failed to clone stream");
                    let mut reader = BufReader::new(stream);
                    match writer.write_all(format!("STREAM udp://{} {}", udp_bind_adr, tickers).as_bytes()){
                        Ok(_) => {
                            writer.flush()?;
                            let mut result = String::new();
                            if let Ok(_) = reader.read_line(&mut result) && result.contains("OK") {
                                let udp = socket.socket.try_clone()?;
                                let is_running_ping = socket.is_running_ping.clone();
                                let server_adr = socket.remote_add.clone();
                                thread::spawn(move || {
                                    QuoteStreamClient::thread_ping_quote_server(
                                        udp,
                                        server_adr,
                                        is_running_ping
                                    )
                                });
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
            if !socket.is_running_ping.load(SeqCst) {
                is_connected = false;
            }
            let mut quote: Vec<u8> = Vec::new();
            match socket.socket.recv_from(&mut quote) {
                Ok((size, src)) => {
                    if size > 0 {
                        if let Some(quote) = StockQuote::from_string(String::from_utf8_lossy(&quote[..size]).as_ref()) {
                            println!("{:?}", quote);
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