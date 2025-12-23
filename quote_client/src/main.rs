#![warn(missing_docs)]

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use quote_lib::quote::stockquote::StockQuote;
use crate::client::QuoteStreamClient;
use clap::{Arg, Command};
mod client;
mod error;

fn main() {
   /* let matches = Command::new("quote-client")
        .version("0.1.0")
        .about("Demo quote stream client")
        .arg(
            Arg::new("server")
                .short('s')
                .long("server")
                .help("Destination host address: host:port")
                .required(true)
        )
        .arg(
            Arg::new("udp")
                .short('u')
                .long("udp")
                .help("Client udp address: host:port")
                .required(true)
        )
        .arg(
            Arg::new("tickets")
                .short('t')
                .long("tickets")
                .help("File path tickets file")
                .required(true)
        )
        .get_matches();
    let host = matches.get_one::<String>("server");
    let udp = matches.get_one::<String>("udp");
    let tickets_file = matches.get_one::<String>("tickets");*/
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
