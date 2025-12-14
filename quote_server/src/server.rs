use std::collections::HashSet;
use std::io::{BufRead, Read};
use std::io::BufReader;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use crossbeam_channel::Receiver;
use std::thread;
use std::thread::{JoinHandle};
use crossbeam_channel::unbounded;
use quote_lib::quote::stockquote::StockQuote;
use crate::error::QuoteStreamServerError;
use crate::quote::volume_generator::{QuoteGenerator};
use crate::quote::quote_stream::{QuoteStream};

pub(crate) struct QuoteServer{
 }

impl QuoteServer {
    fn handle_client(stream: TcpStream, receiver: Receiver<StockQuote>) {
        // клонируем stream: один экземпляр для чтения (обёрнут в BufReader), другой — для записи
        let mut writer = stream.try_clone().expect("failed to clone stream");
        let mut reader = BufReader::new(stream);

        // send initial prompt
        let _ = writer.write_all(b"Welcome to the Vault!\n");
        let _ = writer.flush();
        let mut line = String::new();

        loop {
            line.clear();
            // read_line ждёт '\n' — nc отправляет строку по нажатию Enter
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
                            let thread = thread::scope(|s|{
                                s.spawn(||{
                                    let quote_stream = QuoteStream::thread_stream(
                                        "0.0.0.0:12345",
                                        receiver.clone(),
                                        "AAPL,MSFT,TSLA"
                                    ).expect("");
                                });
                            });
                            "OK\n".to_string()
                        }

                        Some("RESTREAM") => {
                            "OK\n".to_string()
                        }

                        Some("STOP") => {
                            "OK\n".to_string()
                        }

                        _ => "BAD\n".to_string(),
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

    pub fn run_quote_server<R: Read>(r: &mut R) -> Result<(), QuoteStreamServerError>{
        if let Ok(tickers) = StockQuote::get_quotes(r) {
            let (sender, receiver) = unbounded::<StockQuote>();
            thread::scope(|s| {
                s.spawn(|| {
                    QuoteGenerator::thread_generate(sender, &tickers).expect("TODO: panic message");
                });
            });
            let listener = TcpListener::bind("127.0.0.1:7878")?;
            println!("Server listening on port 7878");
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let value = receiver.clone();
                        thread::spawn(move || {
                            QuoteServer::handle_client(stream, value);
                        });
                    }
                    Err(e) => return Err(QuoteStreamServerError::BadCreateTcpStream(e.to_string()))
                }
            }
        }
        Ok(())
    }
}
