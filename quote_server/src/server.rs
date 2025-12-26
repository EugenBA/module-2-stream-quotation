use std::io::{BufRead, Read};
use std::io::BufReader;
use std::io::Write;
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::str::SplitWhitespace;
use std::sync::{Arc, Mutex};
use crossbeam_channel::{bounded, Receiver};
use std::thread;
use std::thread::JoinHandle;
use quote_lib::quote::stockquote::StockQuote;
use crate::error::QuoteStreamServerError;
use crate::quote::volume_generator::{QuoteGenerator};
use crate::quote::quote_stream::{QuoteStream, QuoteStreamResult};
use log;

#[derive(Default)]
pub(crate) struct QuoteServer{
    thread: Option<JoinHandle<Result<QuoteStreamResult, QuoteStreamServerError>>>,
    thread_state: Option<Arc<Mutex<QuoteServerThreadState>>>,
    subscribe_tickers: Arc<Mutex<Vec<StockQuote>>>,
 }


#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum QuoteServerThreadState{
    Running,
    Cancelled,
    Stopped
}



impl QuoteServer {

    fn parse_cmd_stream(split_whitespace: &mut SplitWhitespace) -> Option<(String, String)>{
        if let Some(client_adr) = split_whitespace.next() && let Some(tickers) =
            split_whitespace.next(){
            return Some((client_adr.replace("udp://",""), tickers.to_string()))
        }
        None
    }

    fn start_quote_stream(&mut self, udp_socket: UdpSocket,
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
            self.thread_state = Some(thread_state_stream.clone());
            self.thread = Some(thread::spawn(move || {
                return QuoteStream::thread_stream(
                    udp_socket,
                    &client_adr,
                    receiver,
                    subscribe_tickers,
                    thread_state_stream
                )
            }));
            "OK Stream\n".to_string()
        }
        else { "Error command stream\n".to_string() }

    }
    fn stop_quote_stream(&mut self) {
        if let Some(thread_state) = &self.thread_state &&
            let Ok(mut state) = thread_state.lock() {
            *state = QuoteServerThreadState::Cancelled;
        }
    }

    fn handle_client(&mut self, udp_socket: UdpSocket, stream: TcpStream, receiver: Receiver<StockQuote>) {
        // клонируем stream: один экземпляр для чтения (обёрнут в BufReader), другой — для записи
        let mut writer = stream.try_clone().expect("failed to clone stream tcp");
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
                        Some("STREAM")  | Some("RESTREAM")=> {
                            let udp = udp_socket.try_clone().expect("failed to clone stream udp");
                            log::info!("start stream");
                            self.stop_quote_stream();
                            self.start_quote_stream(udp, parts, receiver.clone())
                        }
                        Some("STOP") => {
                            if let Some(_) = self.thread_state{
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
                    log::error!("error tcp handle");
                    break;
                }
            }
        }
        self.stop_quote_stream();
    }

    pub fn run_quote_server<R: Read>(r: &mut R, tcp_bind: &str, udp_bind: &str) -> Result<(), QuoteStreamServerError>{
        if let Ok(tickers) = StockQuote::get_tickers(r) {
            let (sender, receiver) = bounded::<StockQuote>(tickers.len());
            let _ = thread::scope(|s| {
                s.spawn(|| {
                    QuoteGenerator::thread_generate(sender, &tickers)
                        .expect("Generator quote run error");
                });
                s.spawn(||{
                    let listener = TcpListener::bind(tcp_bind)?;
                    let udp_bind = UdpSocket::bind(udp_bind)?;
                    log::info!("{}", format!("server listening on: {}", tcp_bind.to_string()));
                    for stream in listener.incoming() {
                        match stream {
                            Ok(stream) => {
                                let value = receiver.clone();
                                let udb_bind_adr = udp_bind.try_clone()?;
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
            return Err(QuoteStreamServerError::BadCreateTcpStream("No read tickers".to_string()))
        }
        Ok(())
    }
}
