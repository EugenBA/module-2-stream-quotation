use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::error::QuoteStreamServerError;
use crossbeam_channel::Receiver;
use quote_lib::quote::stockquote::StockQuote;
use crate::server::QuoteServerThreadState;
use log;

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

    pub(crate) fn thread_update_tickers(r: Receiver<StockQuote>,
                                        tickers: Arc<Mutex<Vec<StockQuote>>>,
                                        thread_state: Arc<Mutex<QuoteServerThreadState>>) -> Result<QuoteStreamResult, QuoteStreamServerError> {
        if let Ok(mut state) = thread_state.lock() {
            *state = QuoteServerThreadState::Running;
        } else {
            return Err(QuoteStreamServerError::ChangeThreadStateError("Error change thread update state".to_string()));
        }
        log::debug!("thread update tickers: run");
        loop {
            if let Ok(quote) = r.recv() {
                if let Ok(mut tickers_guard) = tickers.lock() {
                    tickers_guard.iter_mut().for_each(|ticker| {
                        if ticker.ticker == quote.ticker {
                            ticker.price = quote.price;
                            ticker.volume = quote.volume;
                            ticker.timestamp = quote.timestamp;
                        }
                    });
                }
            }
            if let Ok(mut state) = thread_state.lock() {
                if *state == QuoteServerThreadState::Cancelled || *state == QuoteServerThreadState::Stopped {
                    *state = QuoteServerThreadState::Stopped;
                    break;
                }
            }
        }
        log::debug!("thread update tickers: stop");
        Ok(QuoteStreamResult::Canceled)
    }

    pub(crate) fn thread_stream(udp_bind_adr: &str, client_adr: &str, receiver: Receiver<StockQuote>,
                                tickers: Arc<Mutex<Vec<StockQuote>>>,
                                thread_state: Arc<Mutex<QuoteServerThreadState>>) -> Result<QuoteStreamResult, QuoteStreamServerError> {
        let mut socket = Self::new(udp_bind_adr)?;
        socket.socket.set_read_timeout(Some(Duration::from_secs(UDP_READ_TIMEOUT_SECOND)))?;
        if let Ok(mut state) = thread_state.lock() {
            *state = QuoteServerThreadState::Running;
        } else {
            return Err(QuoteStreamServerError::ChangeThreadStateError("Error change thread stream state".to_string()));
        }
        log::debug!("thread stream quotes: run");
        let subscribe_tickers_update = tickers.clone();
        let thread_state_updater = Arc::new(Mutex::new(QuoteServerThreadState::Stopped));
        let thread_state_ticker_update = thread_state_updater.clone();
        let _ = thread::spawn(move || {
            return QuoteStream::thread_update_tickers(receiver, subscribe_tickers_update,
                                                      thread_state_ticker_update);
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
                Ok((size, _)) => {
                    if size > 0 && String::from_utf8_lossy(&ping[..size]).contains("PING") {
                        socket.keep_alive_timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)?
                            .as_secs()
                    }
                }
                Err(_) => {
                    if SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() -
                        socket.keep_alive_timestamp > 5 {
                        if let Ok(mut state) = thread_state.lock() {
                            *state = QuoteServerThreadState::Cancelled;
                        }
                    }
                }
            }
            if let Ok(state) = thread_state.lock() {
                if *state == QuoteServerThreadState::Cancelled || *state == QuoteServerThreadState::Stopped {
                    break;
                }
            }
        }
        loop {
            if let Ok(mut state) = thread_state_updater.lock() {
                *state = QuoteServerThreadState::Cancelled;
                break;
            }
            log::debug!("wait stop updater...");
            thread::sleep(Duration::from_secs(1));
        }
        log::debug!("thread stream quotes: stop");
        Ok(QuoteStreamResult::Canceled)
    }
}
