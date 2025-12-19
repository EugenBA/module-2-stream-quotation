use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::error::QuoteStreamServerError;
use crossbeam_channel::Receiver;
use quote_lib::quote::stockquote::StockQuote;

pub(crate) struct QuoteStream {
    socket: UdpSocket,
    keep_alive_timestamp: u64
}

pub(crate) enum QuoteStreamResult {
    Canceled,
}
const UDP_READ_TIMEOUT_SECOND: u64 = 4;

impl QuoteStream {
    pub fn new(bind_adr: &str) -> Result<Self, QuoteStreamServerError> {
        Ok(Self {
            socket: UdpSocket::bind(bind_adr)?,
            keep_alive_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs(),
        })
    }

    pub fn thread_stream(udp_bind_adr: &str, client_adr: &str,
                         r: Receiver<StockQuote>, tickers: &str,
                         is_running: Arc<AtomicBool>,
                         cancel_token: Arc<AtomicBool>) -> Result<QuoteStreamResult, QuoteStreamServerError> {
        let mut socket = Self::new(udp_bind_adr)?;
        socket.socket.set_read_timeout(Some(Duration::from_secs(UDP_READ_TIMEOUT_SECOND)))?;
        let ticks = tickers.split(",").collect::<Vec<&str>>();
        is_running.store(true, SeqCst);
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
                            .duration_since(UNIX_EPOCH)?
                            .as_secs()
                    }
                }
                Err(_) => {
                    if SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() -
                        socket.keep_alive_timestamp > 5 {
                        break;
                    }
                }
            }
            if cancel_token.load(SeqCst) {
                break;
            }
        }
        is_running.store(false, SeqCst);
        if cancel_token.load(SeqCst) {
            cancel_token.store(false, SeqCst);
            Ok(QuoteStreamResult::Canceled)
        }
        else {
            Err(QuoteStreamServerError::KeepAliveTimeoutError("KeepAlive timeout error".to_string()))
        }
    }
}
