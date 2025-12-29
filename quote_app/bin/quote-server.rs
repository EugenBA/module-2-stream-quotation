#![warn(missing_docs)]

//! Приложение - сервер стриминнга котировк
//!
//! Обрабатывает запрос клиента в отдельном потоке, стримит котировки по UDP
#![allow(unused_imports, unused_variables)]
#[path="../src/quote.rs"]
mod quote;
#[path="../src/server.rs"]
mod server;
#[path="../src/error.rs"]
mod error;
#[path="../src/parsecli.rs"]
mod parsecli;
#[path="../src/logger.rs"]
mod logger;


use std::fs::File;
use std::io::BufReader;
#[cfg(feature = "server")]
use crate::server::{QuoteServer};
use crate::parsecli::CliArgs;
use crate::logger::setup_logger;

fn main() {
    #[cfg(feature = "server")]{
        let cli_args = CliArgs::get_cli_args();
        if let Some(arg) = cli_args {
            setup_logger(arg.log_level, &arg.file_log);
            let mut reader = BufReader::new(File::open(arg.tickers_file).unwrap());
            if let Err(quote_server) =
                QuoteServer::run_quote_server(&mut reader, &arg.server_addr, &arg.udp_addr) {
                println!("Error: {}", quote_server);
            }
        } else {
            println!("Error parsing argument");
        }
    }
}
