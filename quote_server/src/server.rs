use std::io::{BufRead, Read};
use std::io::BufReader;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::str::SplitWhitespace;
use std::sync::{Arc};
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
    thread: Option<JoinHandle<Result<QuoteStreamResult, QuoteStreamServerError>>>,
    is_running: Arc<AtomicBool>,
    cancel_token: Arc<AtomicBool>
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
        if let Some((client_adr, tickets)) = QuoteServer::parse_cmd_stream(&mut cmd)
        {
            let is_run = self.is_running.clone();
            let cancel_token = self.cancel_token.clone();
            self.thread = Some(thread::spawn(move || {
                return QuoteStream::thread_stream(
                    &udp_bind_adr,
                    &client_adr,
                    receiver,
                    &tickets,
                    is_run,
                    cancel_token
                )
            }));
            "OK Stream\n".to_string()
        }
        else { "Error command stream\n".to_string() }

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
            // read_line ждёт '\n' — nc отправляет строку по нажатию Enter
            if let Some(thread) = &self.thread{
                if self.is_running.load(SeqCst) {
                    let _ =writer.write_all(b"is_running: true\n");
                    let _ =writer.flush();
                }
            }
            match reader.read_line(&mut line) {
                Ok(0) => {
                    // EOF — клиент закрыл соединение
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
                            /*if let Some((client_adr, tickets)) = QuoteServer::parse_cmd_stream(&mut parts)
                            {
                                let value = receiver.clone();
                                let udp_bind = udp_bind_adr.clone();
                                let is_run = self.is_running.clone();
                                self.thread = Some(thread::spawn(move || {
                                        return QuoteStream::thread_stream(
                                            &udp_bind,
                                            &client_adr,
                                            value.clone(),
                                            &tickets,
                                            is_run
                                        )
                                    }));
                                "OK\n".to_string()
                            }
                            else { "Error command\n".to_string() }*/
                        }

                        Some("RESTREAM") => {
                            while self.is_running.load(SeqCst) {
                                self.cancel_token.store(true, SeqCst);
                            }
                            self.start_quote_stream(udp_bind_adr.clone(), parts, receiver.clone())
                        }

                        Some("STOP") => {
                            while self.is_running.load(SeqCst) {
                                self.cancel_token.store(true, SeqCst);
                            }
                            "OK Stop\n".to_string()
                        }

                        _ => "Error command\n".to_string(),
                    };

                    // отправляем ответ и снова показываем prompt
                    let _ = writer.write_all(response.as_bytes());
                    let _ = writer.flush();
                }
                Err(_) => {
                    // ошибка чтения — закрываем
                    return;
                }
            }
        }
    }

    pub fn run_quote_server<R: Read>(r: &mut R, tcp_bind: &str, udp_bind: &str) -> Result<(), QuoteStreamServerError>{
        if let Ok(tickers) = StockQuote::get_quotes(r) {
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
