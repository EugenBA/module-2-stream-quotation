use std::net::UdpSocket;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::QuoteStreamServerError;
use crossbeam_channel::Receiver;
use quote_lib::quote::stockquote::StockQuote;

pub(crate) struct QuoteStream {
    socket: UdpSocket,
    keep_alive_timestamp: u64
}

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

    fn send_quote(&self, quote: &str) -> Result<(), QuoteStreamServerError>{
        self.socket.send(quote.as_bytes())?;
        Ok(())
    }

    pub fn thread_stream(bind_adr: &str, r: Receiver<StockQuote>, tickers: &str) -> Result<(), QuoteStreamServerError> {
        let mut socket = Self::new(bind_adr)?;
        let ticks = tickers.split(",").collect::<Vec<&str>>();
        loop {
            if let Ok(quote) = r.recv() {
                if ticks.contains(&quote.ticker.as_str()) {
                    socket.socket.send(&quote.to_bytes())?;
                }
            }
            let mut ping: Vec<u8> = Vec::new();
            if let Ok(size) =  socket.socket.recv(&mut ping)  {
                if size > 0 && String::from_utf8_lossy(&ping[..size]).contains("PING"){

                    socket.keep_alive_timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                }
            }
            if SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() -
                socket.keep_alive_timestamp > 300 {
                break;
            }
        }
        Err(QuoteStreamServerError::KeepAliveTimeoutError("Timeout client".to_string()))
    }

}