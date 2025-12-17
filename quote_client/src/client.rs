use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use quote_lib::quote::stockquote::StockQuote;
use crate::error::QuoteClientError;

struct QuoteStreamClient {
    socket: UdpSocket,
    is_running_ping: Arc<AtomicBool>
}
const UDP_READ_TIMEOUT_SECOND: u64 = 4;

impl QuoteStreamClient {
    pub fn new(bind_adr: &str) -> Result<Self, QuoteClientError> {
        Ok(Self {
            socket: UdpSocket::bind(bind_adr)?,
            is_running_ping: Arc::new(AtomicBool::new(false))
        })
    }

    fn connect_quote_server(server_adr: &str) -> Result<(), QuoteClientError>{

    }

    fn thread_ping_quote_server(&self, server_adr: &str, is_running: Arc<AtomicBool>){

    }

    pub fn get_quote_stream(udp_bind_adr: &str,
                         is_running: Arc<AtomicBool>) -> Result<(), QuoteClientError> {
        let socket = Self::new(udp_bind_adr)?;
        socket.socket.set_read_timeout(Some(Duration::from_secs(UDP_READ_TIMEOUT_SECOND)))?;
        is_running.store(true, SeqCst);
        loop {
            let mut quote: Vec<u8> = Vec::new();
            match socket.socket.recv_from(&mut quote) {
                Ok((size, src)) => {
                    if size > 0 {
                       if  let Some(quote) = StockQuote::from_string(String::from_utf8_lossy(&quote[..size]).as_ref()){
                            println!("{:?}", quote);
                        }
                    }
                }
                Err(e) => {
                    println!("Error quote client: {}", e.to_string());
                }
            }
        }
    }
}