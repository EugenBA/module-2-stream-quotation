#![warn(missing_docs)]
//! Приложение - клиент для получения стриминнга котировк
//!
//! Производит запрос к среверу и по UDP получает поток данных по котировкам.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use quote_lib::quote::stockquote::StockQuote;
use crate::client::QuoteStreamClient;
use log::{warn, LevelFilter};
use clap::{Arg, Command};
use env_logger::{Builder, Target};
use std::io::Write;

mod client;
mod error;

fn setup_logger(level: LevelFilter) {
    let log_file = File::create("quote-client.log").expect("Error create log file");
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}:{} - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .target(Target::Pipe(Box::new(log_file)))
        .filter(None, level) // Уровень по умолчанию
        .write_style(env_logger::WriteStyle::Always) // Всегда использовать цвета
        .init();
}


fn main() {
    let matches = Command::new("quote-client")
        .version("0.1.0")
        .about("Demo quote stream client")
        .arg(
            Arg::new("server-addr")
                .short('s')
                .long("server-addr")
                .help("Destination host address: host:port")
                .required(true)
        )
        .arg(
            Arg::new("udp-port")
                .short('u')
                .long("udp-port")
                .help("Client udp port: port")
                .required(true)
        )
        .arg(
            Arg::new("tickers-file")
                .short('t')
                .long("tickers-file")
                .help("File path tickers file")
                .required(true)
        )
        .arg(
            Arg::new("log-level")
                .short('l')
                .long("log-level")
                .help("Log level")
                .default_value("INFO")
                .required(false)
        )
        .get_matches();
    let server_addr = matches.get_one::<String>("server-addr");
    let udp_port = matches.get_one::<String>("udp-port");
    let tickers_file = matches.get_one::<String>("tickers-file");
    let log_level = matches.get_one::<String>("log-level");
    if let Some(server_addr) = server_addr &&
        let Some(udp_port) = udp_port && let Some(tickers_file) = tickers_file &&
        let Some(log_level) = log_level{
        if !Path::new(tickers_file).exists() {
            eprintln!("File {} not exists", tickers_file);
            return;
        }
        let level = {
            match log_level.as_ref() {
                "DEBUG" => LevelFilter::Debug,
                "ERROR" => LevelFilter::Error,
                "WARN" => LevelFilter::Warn,
                _ => LevelFilter::Info,
            }
        };
        setup_logger(level);
        let host_port: Vec<&str> = server_addr.split(":").collect();
        if host_port.len() < 2{
            eprintln!("Error address server");
        }
        let udp_addr = host_port[0].to_owned()  + ":" + udp_port;
        let mut reader = BufReader::new(File::open(tickers_file).unwrap());
        let tickers = StockQuote::get_tickers_string_from_file(&mut reader).unwrap();
        let mut quote_stream_client = QuoteStreamClient::default();
        if let Err(e) = quote_stream_client.get_quote_stream(&udp_addr,
                                                             server_addr, tickers)
        {
            println!("Error: {}", e);
        }
    }
    else {
        print!("Bad command args")
    }
}
