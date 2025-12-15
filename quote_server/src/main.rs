#![warn(missing_docs)]
mod quote;
mod server;
mod error;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use crate::server::{QuoteServer};

fn main() {
    let in_file = "/home/eugen/RustroverProjects/module-2-stream-quotation/tickets/tickets.txt";
    if !Path::new(&in_file).exists() {
        eprintln!("Файл {} не существует", in_file);
        return;
    }
    let url_bind = "127.0.0.1:9120".to_string();
    let mut reader = BufReader::new(File::open(in_file).unwrap());
    if let Err(quote_server) =
        QuoteServer::run_quote_server(&mut reader, &url_bind){
        println!("Error: {}", quote_server);
    }
}
