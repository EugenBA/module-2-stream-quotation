#![warn(missing_docs)]

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

fn setup_logger() {
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
        .filter(None, LevelFilter::Info) // Уровень по умолчанию
        .target(Target::Stdout) // Вывод в stdout вместо stderr
        .write_style(env_logger::WriteStyle::Always) // Всегда использовать цвета
        .init();
}


fn main() {
   /* let matches = Command::new("quote-client")
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
                .help("Client udp address: host:port")
                .required(true)
        )
        .arg(
            Arg::new("tickers")
                .short('t')
                .long("tickers")
                .help("File path tickets file")
                .required(true)
        )
        .get_matches();
    let host = matches.get_one::<String>("server");
    let udp = matches.get_one::<String>("udp");
    let tickers_file = matches.get_one::<String>("tickers-file");*/
    setup_logger();
    let host = Some("127.0.0.1:8120");
    let udp = Some("127.0.0.1:55500");
    let tickets_file = Some("/home/eugen/RustroverProjects/module-2-stream-quotation/tickets/tickets_request.txt");
    if let Some(host) = host && let Some(udp) = udp && let Some(tickets_file) = tickets_file{
        if !Path::new(tickets_file).exists() {
            eprintln!("Файл {} не существует", tickets_file);
            return;
        }
        let mut reader = BufReader::new(File::open(tickets_file).unwrap());
        let tickers = StockQuote::get_tickers_string_from_file(&mut reader).unwrap();
        let mut quote_stream_client = QuoteStreamClient::default();
        if let Err(e) = quote_stream_client.get_quote_stream(udp, host, tickers)
        {
            println!("Error: {}", e);
        }
    }
    else {
        print!("Bad command args")
    }
}
