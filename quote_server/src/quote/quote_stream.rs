use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::thread;
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
const UDP_SEND_PERIOD: u64 = 2;

impl QuoteStream {
    pub fn new(bind_adr: &str) -> Result<Self, QuoteStreamServerError> {
        Ok(Self {
            socket: UdpSocket::bind(bind_adr)?,
            keep_alive_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs(),
        })
    }

    fn thread_update_tickers(r: Receiver<StockQuote>,
                             tickers: Arc<Mutex<Vec<StockQuote>>>,
                             is_running: Arc<AtomicBool>,
                             cancel_token: Arc<AtomicBool>){
        loop {
            is_running.store(true, SeqCst);
            if let Ok(quote) = r.recv() {
                if let Ok(mut tickers_guard) = tickers.lock(){
                    tickers_guard.iter_mut().for_each(|ticker| {
                        if ticker.ticker == quote.ticker {
                            ticker.price = quote.price;
                            ticker.volume = quote.volume;
                            ticker.timestamp = quote.timestamp;
                        }
                    });
                }
            }
            if cancel_token.load(SeqCst) {
                break;
            }

        }
        is_running.store(false, SeqCst);
    }

    pub fn thread_stream(udp_bind_adr: &str, client_adr: &str,
                         r: Receiver<StockQuote>, tickers: Arc<Mutex<Vec<StockQuote>>>,
                         is_running: Arc<AtomicBool>,
                         cancel_token: Arc<AtomicBool>) -> Result<QuoteStreamResult, QuoteStreamServerError> {
        let mut socket = Self::new(udp_bind_adr)?;
        socket.socket.set_read_timeout(Some(Duration::from_secs(UDP_READ_TIMEOUT_SECOND)))?;
        is_running.store(true, SeqCst);
        let is_running_update_tickers : Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let cancel_token_updater_tickers : Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let ticker_update = tickers.clone();
        let is_run_update_ticker = is_running_update_tickers.clone();
        let cancel_update_ticker = cancel_token_updater_tickers.clone();
        let thread = thread::spawn(move || {
            return QuoteStream::thread_update_tickers(r, ticker_update,
                                                      is_run_update_ticker,
                                                      cancel_update_ticker);
        });
        loop {
            thread::sleep(Duration::from_secs(UDP_SEND_PERIOD));
            if let Ok(tickers_guard) = tickers.lock() {
                tickers_guard.iter().for_each(|tickers| {
                      let _ = socket.socket.send_to(&tickers.to_bytes(), client_adr);
                });
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
                while is_running_update_tickers.load(SeqCst) {
                    cancel_token_updater_tickers.store(true, SeqCst);
                }
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
