
use crate::error::servererror::QuoteStreamServerError;
use crate::server::QuoteServerThreadState;
use crossbeam_channel::Receiver;
use log;
use quote_lib::quote::stockquote::StockQuote;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) struct QuoteStream {
    socket: UdpSocket,
    keep_alive_timestamp: u64,
}

pub(crate) enum QuoteStreamResult {
    Canceled,
}
const UDP_READ_TIMEOUT_SECOND: u64 = 6;
const UDP_SEND_PERIOD: u64 = 2;
const PING_READ_TIMEOUT: u64 = 5;

impl QuoteStream {
    pub fn new(udp_socket: UdpSocket) -> Result<Self, QuoteStreamServerError> {
        Ok(Self {
            socket: udp_socket,
            keep_alive_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        })
    }

    fn thread_update_tickers(
        r: Receiver<StockQuote>,
        tickers: Arc<Mutex<Vec<StockQuote>>>,
        thread_state: Arc<Mutex<QuoteServerThreadState>>,
    ) -> Result<QuoteStreamResult, QuoteStreamServerError> {
        //Метод обновления котировок
        //Работает циклически и обновляет данные котировок
        if let Ok(mut state) = thread_state.lock() {
            *state = QuoteServerThreadState::Running;
        } else {
            return Err(QuoteStreamServerError::ChangeThreadStateError(
                "Error change thread update state".to_string(),
            ));
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
            } else {
                return Err(QuoteStreamServerError::ReceiveQuoteError(
                    "Error receiving quote".to_string(),
                ));
            }
            if let Ok(mut state) = thread_state.lock() {
                if *state == QuoteServerThreadState::Cancelled
                    || *state == QuoteServerThreadState::Stopped
                {
                    *state = QuoteServerThreadState::Stopped;
                    break;
                }
            }
        }
        log::debug!("thread update tickers: stop");
        Ok(QuoteStreamResult::Canceled)
    }

    pub(crate) fn thread_stream(
        udp_bind_adr: UdpSocket,
        client_adr: &str,
        receiver: Receiver<StockQuote>,
        tickers: Arc<Mutex<Vec<StockQuote>>>,
        thread_state: Arc<Mutex<QuoteServerThreadState>>,
    ) -> Result<QuoteStreamResult, QuoteStreamServerError> {
        //Метод стриммиинга - отправляет данные клиенту, запускает поток обновления данных
        //отсанавливает поток обновления котировк в случает не получаени данных ping от клиента
        let mut socket = Self::new(udp_bind_adr)?;
        socket
            .socket
            .set_read_timeout(Some(Duration::from_secs(UDP_READ_TIMEOUT_SECOND)))?;
        if let Ok(mut state) = thread_state.lock() {
            *state = QuoteServerThreadState::Running;
        } else {
            return Err(QuoteStreamServerError::ChangeThreadStateError(
                "Error change thread stream state".to_string(),
            ));
        }
        log::debug!("thread stream quotes: run");
        let subscribe_tickers_update = tickers.clone();
        let thread_state_updater = Arc::new(Mutex::new(QuoteServerThreadState::Stopped));
        let thread_state_ticker_update = thread_state_updater.clone();
        let _ = thread::spawn(move || {
            return QuoteStream::thread_update_tickers(
                receiver,
                subscribe_tickers_update,
                thread_state_ticker_update,
            );
        });
        //основной цикл отправления данных клиенту
        loop {
            if let Ok(tickers_guard) = tickers.lock() {
                tickers_guard.iter().for_each(|tickers| {
                    let _ = socket.socket.send_to(&tickers.to_bytes(), client_adr);
                });
            }
            let mut ping = [0u8; 1024];
            match socket.socket.recv_from(&mut ping) {
                Ok((size, src)) => {
                    if size > 0
                        && String::from_utf8_lossy(&ping[..size]).contains("PING")
                        && client_adr == src.to_string()
                    {
                        socket.keep_alive_timestamp =
                            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                    }
                }
                Err(e) => {
                    log::error!("Error receiving ping message from socket: {}", e);
                }
            }
            if SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() - socket.keep_alive_timestamp
                > PING_READ_TIMEOUT
            {
                if let Ok(mut state) = thread_state.lock() {
                    *state = QuoteServerThreadState::Cancelled;
                }
            }
            if let Ok(state) = thread_state.lock() {
                if *state == QuoteServerThreadState::Cancelled
                    || *state == QuoteServerThreadState::Stopped
                {
                    break;
                }
            }
            thread::sleep(Duration::from_secs(UDP_SEND_PERIOD));
        }
        //ожидание остановки потока обновления котировок
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

#[cfg(test)]
mod test {
    use super::*;
    use crossbeam_channel::bounded;
    #[test]
    fn test_thread_update_tickers() {
        let (sender, receiver) = bounded::<StockQuote>(5);
        let thread_stop;
        let initial_quote = StockQuote {
            ticker: "A".to_string(),
            price: 10.0,
            volume: 10,
            timestamp: 10,
        };

        let tickers = Arc::new(Mutex::new(vec![initial_quote.clone()]));
        let thread_state = Arc::new(Mutex::new(QuoteServerThreadState::Stopped));

        let tickers_clone = tickers.clone();
        let thread_state_clone = thread_state.clone();

        let updated_quote = StockQuote {
            ticker: "A".to_string(),
            price: 10.0,
            volume: 2000,
            timestamp: 2000,
        };
        sender.send(updated_quote.clone()).unwrap();

        let handle = thread::spawn(move || {
            QuoteStream::thread_update_tickers(receiver, tickers_clone, thread_state_clone)
        });

        thread::sleep(Duration::from_millis(50));

        // Verify update
        {
            let tickers_guard = tickers.lock().unwrap();
            assert_eq!(tickers_guard[0].price, 10.0);
            assert_eq!(tickers_guard[0].volume, 2000);
            assert_eq!(tickers_guard[0].timestamp, 2000);
        }

        {
            let mut state = thread_state.lock().unwrap();
            *state = QuoteServerThreadState::Stopped;
        }
        loop {
            if let Ok(mut state) = thread_state.lock() {
                *state = QuoteServerThreadState::Cancelled;
                thread_stop = true;
                break;
            }
        }
        assert!(thread_stop);
    }
}
