#![warn(missing_docs)]

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use quote_lib::quote::stockquote::StockQuote;
use crate::client::QuoteStreamClient;

mod client;
mod error;

fn main() {
    let in_file = "/mnt/ssd_data/RustProject/module2-stream-quotation/tickets/tickets_request.txt";
    if !Path::new(&in_file).exists() {
        eprintln!("Файл {} не существует", in_file);
        return;
    }
    let server_addr = "127.0.0.1:7878".to_string();
    let udp_bind = "127.0.0.1:55500".to_string();
    let mut reader = BufReader::new(File::open(in_file).unwrap());
    let tickers = StockQuote::get_tickers_string_from_file(&mut reader).unwrap();
    if let Err(e) = QuoteStreamClient::get_quote_stream(&udp_bind, &server_addr, tickers)
    {
        println!("Error: {}", e);
    }
}
