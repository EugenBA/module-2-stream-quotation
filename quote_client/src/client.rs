use std::io::BufReader;
use std::net::{TcpStream, UdpSocket};
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use socket2::{Domain, Protocol, Socket, Type};
use quote_lib::quote::stockquote::StockQuote;
use crate::error::QuoteClientError;

struct QuoteStreamClient {
    socket: UdpSocket,
    is_running_ping: Arc<AtomicBool>,
    remote_add: Arc<RwLock<String>>
}
const UDP_READ_TIMEOUT_SECOND: u64 = 4;

impl QuoteStreamClient {
    pub fn new(bind_adr: &str) -> Result<Self, QuoteClientError> {
        Ok(Self {
            socket: UdpSocket::bind(bind_adr)?,
            is_running_ping: Arc::new(AtomicBool::new(false))
        })
    }

    fn connect(addr: &SocketAddr) -> Result<TcpStream, QuoteStreamClient> {
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;

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

        socket.connect(&addr.clone().into())?;
        let stream: TcpStream = socket.into();

        // тайм-аут на чтение
        stream.set_read_timeout(Some(Duration::from_secs(3)))?;

        Ok(stream)
    }

    fn connect_quote_server(server_adr: &str) -> Result<(), QuoteClientError> {
        loop {
            // пробуем подключиться
            match QuoteStreamClient::connect(&addr) {
                Ok(stream) => {
                    println!("Connected to server!");
                    match handle_connection(stream) {
                        ConnectionResult::Exit => break,
                        ConnectionResult::Lost => {
                            println!("Connection lost. Reconnecting in 2s...");
                            thread::sleep(Duration::from_secs(2));
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Connect failed: {}. Retrying in 2s...", err);
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }
    }

    fn thread_ping_quote_server(&self, server_adr: &str, is_running: Arc<AtomicBool>) {}

    pub fn get_quote_stream(&mut self, udp_bind_adr: &str, server_adr: &str) -> Result<(), QuoteClientError> {
        let socket = Self::new(udp_bind_adr)?;
        socket.socket.set_read_timeout(Some(Duration::from_secs(UDP_READ_TIMEOUT_SECOND)))?;
        let mut is_connected = false;
        loop {
            if !is_connected{
                while self.is_running_ping.load(SeqCst) {
                    self.is_running_ping.store(true, SeqCst);
                }
                if let Ok(stream) = QuoteStreamClient::connect(server_adr) {
                    let mut writer = stream.try_clone().expect("failed to clone stream");
                    let mut reader = BufReader::new(stream);

                }
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