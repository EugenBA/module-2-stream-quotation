use std::io::{BufRead, Read};
use std::io::BufReader;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::str::SplitWhitespace;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use crossbeam_channel::{bounded, Receiver};
use std::thread;
use std::thread::JoinHandle;
use quote_lib::quote::stockquote::StockQuote;
use crate::error::QuoteStreamServerError;
use crate::quote::volume_generator::{QuoteGenerator};
use crate::quote::quote_stream::{QuoteStream, QuoteStreamResult};

#[derive(Default)]
pub(crate) struct QuoteServer{
    thread: Vec<JoinHandle<Result<QuoteStreamResult, QuoteStreamServerError>>>,
    thread_state: Vec<Arc<Mutex<QuoteServerThreadState>>>,
    subscribe_tickers: Arc<Mutex<Vec<StockQuote>>>,
 }


#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum QuoteServerThreadState{
    Running,
    Cancelled,
    Stopped,
    HalfState
}



impl QuoteServer {

    fn parse_cmd_stream(split_whitespace: &mut SplitWhitespace) -> Option<(String, String)>{
        if let Some(client_adr) = split_whitespace.next() && let Some(tickets) =
            split_whitespace.next(){
            return Some((client_adr.replace("udp://",""), tickets.to_string()))
        }
        None
    }

    fn start_quote_stream(&mut self, udp_bind_adr: String,
                          mut cmd: SplitWhitespace,
                          receiver: Receiver<StockQuote>) -> String {
        if let Some((client_adr, tickers)) = QuoteServer::parse_cmd_stream(&mut cmd)
        {
            let thread_state_stream = Arc::new(Mutex::new(QuoteServerThreadState::Stopped));
            if let Ok(mut tickers_subscribe_lock) = self.subscribe_tickers.lock() {
                tickers_subscribe_lock.clear();
                *tickers_subscribe_lock = StockQuote::get_tickers_subscribe(&tickers);
            }
            else{
                return "Error store subscribe tickers\n".to_string()
            }
            let subscribe_tickers = self.subscribe_tickers.clone();
            self.thread_state.push(thread_state_stream.clone());
            self.thread.push(thread::spawn(move || {
                return QuoteStream::thread_stream(
                    &udp_bind_adr,
                    &client_adr,
                    subscribe_tickers,
                    thread_state_stream
                )
            }));
            let thread_state_ticker_update = Arc::new(Mutex::new(QuoteServerThreadState::Stopped));
            let subscribe_tickers_update = self.subscribe_tickers.clone();
            self.thread_state.push(thread_state_ticker_update.clone());
            self.thread.push(thread::spawn(move || {
                return QuoteStream::thread_update_tickers(receiver, subscribe_tickers_update,
                                                          thread_state_ticker_update);
            }));
            "OK Stream\n".to_string()
        }
        else { "Error command stream\n".to_string() }

    }
    fn stop_quote_stream(&mut self){
        for thread in &mut self.thread_state{
            if let Ok(mut state) = thread.lock() {
                *state = QuoteServerThreadState::Cancelled;
            }
        }
    }
    fn state_thread(&self) -> QuoteServerThreadState{
        let mut count_stoped = 0;
        for thread in &self.thread_state{
            if let Ok(state) = thread.lock() &&
                *state == QuoteServerThreadState::Stopped{
                count_stoped += 1;
            }
        }
        match count_stoped  {
            0 => QuoteServerThreadState::Running,
            _ => {
                if count_stoped == self.thread_state.len() {
                    return QuoteServerThreadState::Stopped;
                }
                QuoteServerThreadState::HalfState
            }
        }
    }

    fn handle_client(&mut self, udp_bind_adr: String, stream: TcpStream, receiver: Receiver<StockQuote>) {
        // клонируем stream: один экземпляр для чтения (обёрнут в BufReader), другой — для записи
        let mut writer = stream.try_clone().expect("failed to clone stream");
        let mut reader = BufReader::new(stream);

        // send initial prompt
        let _ = writer.write_all(b"Welcome to quotation stream!\n");
        let _ = writer.flush();
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    return;
                }
                Ok(_) => {
                    let input = line.trim();
                    if input.is_empty() {
                        let _ = writer.flush();
                        continue;
                    }
                    let mut parts = input.split_whitespace();
                    let response = match parts.next() {
                        Some("STREAM") => {
                            self.start_quote_stream(udp_bind_adr.clone(), parts, receiver.clone())
                        }
                        Some("RESTREAM") => {
                            self.stop_quote_stream();
                            self.start_quote_stream(udp_bind_adr.clone(), parts, receiver.clone())
                        }
                        Some("STOP") => {
                            if self.thread_state.len() > 0 {
                                self.stop_quote_stream();
                                "OK Stop\n".to_string()
                            }
                            else{
                                "Thread not running\n".to_string()
                            }
                        }
                        _ => "Error command\n".to_string(),
                    };

                    // отправляем ответ и снова показываем prompt
                    let _ = writer.write_all(response.as_bytes());
                    let _ = writer.flush();
                }
                Err(_) => {
                    // ошибка чтения — закрываем
                    self.stop_quote_stream();
                    return;
                }
            }
            if self.state_thread() == QuoteServerThreadState::HalfState{
                self.stop_quote_stream();
            }
        }
    }

    pub fn run_quote_server<R: Read>(r: &mut R, tcp_bind: &str, udp_bind: &str) -> Result<(), QuoteStreamServerError>{
        if let Ok(tickers) = StockQuote::get_tickers(r) {
            let (sender, receiver) = bounded::<StockQuote>(tickers.len());
            let thr = thread::scope(|s| {
                s.spawn(|| {
                    QuoteGenerator::thread_generate(sender, &tickers)
                        .expect("Generator quote run error");
                });
                s.spawn(||{
                    let listener = TcpListener::bind(tcp_bind)?;
                    println!("{}", format!("Server listening on: {}", tcp_bind.to_string()));
                    for stream in listener.incoming() {
                        match stream {
                            Ok(stream) => {
                                let value = receiver.clone();
                                let udb_bind_adr = udp_bind.to_string().clone();
                                thread::spawn(move || {
                                    let mut quote_server = QuoteServer::default();
                                    quote_server.handle_client(udb_bind_adr, stream, value);
                                });
                            }
                            Err(e) => return Err(QuoteStreamServerError::BadCreateTcpStream(e.to_string()))
                        }
                    }
                    Ok(())
                });
            });
        }
        else {
            return Err(QuoteStreamServerError::BadCreateTcpStream("No read tickets".to_string()))
        }
        Ok(())
    }
}
