use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::error::QuoteStreamServerError;
use crossbeam_channel::Receiver;
use quote_lib::quote::stockquote::StockQuote;

pub(crate) struct QuoteStream {
    socket: UdpSocket,
    keep_alive_timestamp: u64
}
const UDP_READ_TIMEOUT_SECOND: u64 = 4;

impl QuoteStream {
    pub fn new(bind_adr: &str) -> Result<Self, QuoteStreamServerError> {
        Ok(Self {
            socket: UdpSocket::bind(bind_adr)?,
            keep_alive_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    fn send_quote(&self, quote: &str) -> Result<(), QuoteStreamServerError> {
        self.socket.send(quote.as_bytes())?;
        Ok(())
    }

    pub fn thread_stream(bind_adr: &str, client_adr: &str, r: Receiver<StockQuote>, tickers: &str) -> Result<(), QuoteStreamServerError> {
        let mut socket = Self::new(bind_adr)?;
        socket.socket.set_read_timeout(Some(Duration::from_secs(UDP_READ_TIMEOUT_SECOND)))?;
        let ticks = tickers.split(",").collect::<Vec<&str>>();
        loop {
            if let Ok(quote) = r.recv() {
                if ticks.contains(&quote.ticker.as_str()) {
                    socket.socket.send_to(&quote.to_bytes(), client_adr)?;
                }
            }
            let mut ping: Vec<u8> = Vec::new();
            match socket.socket.recv_from(&mut ping) {
                Ok((size, src)) => {
                    if size > 0 && String::from_utf8_lossy(&ping[..size]).contains("PING") {
                        socket.keep_alive_timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                    }
                }
                Err(_) => {
                    if SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() -
                        socket.keep_alive_timestamp > 5 {
                        break;
                    }
                }
            }
        }
        Err(QuoteStreamServerError::KeepAliveTimeoutError("KeepAlive timeout error".to_string()))
    }
}
