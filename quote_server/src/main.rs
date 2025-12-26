#![warn(missing_docs)]

//! Приложение - сервер стриминнга котировк
//!
//! Обрабатывает запрос клиента в отдельном потоке, стримит котировки по UDP
mod quote;
mod server;
mod error;


use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use crate::server::{QuoteServer};
use clap::{Arg, Command};
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;

fn setup_logger(level: LevelFilter) {
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
        .filter(None, level) // Уровень по умолчанию
        .target(Target::Stdout) // Вывод в stdout вместо stderr
        .write_style(env_logger::WriteStyle::Always) // Всегда использовать цвета
        .init();
}

fn main() {
    let matches = Command::new("quote-server")
        .version("0.1.0")
        .about("Demo quote stream server")
        .arg(
            Arg::new("server-addr")
                .short('s')
                .long("server-addr")
                .help("Local server address: host:port")
                .required(true)
        )
        .arg(
            Arg::new("udp-port")
                .short('u')
                .long("udp-port")
                .help("Server udp bport")
                .required(true)
        )
        .arg(
            Arg::new("tickers-files")
                .short('t')
                .long("tickers")
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
    let tickers_file = matches.get_one::<String>("tickers-files");
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
        //разбираем адресс на данные хост, порт
        if host_port.len() < 2{
            eprintln!("Error address server");
        }
        let udp_addr = host_port[0].to_owned()  + ":" + udp_port;
        let mut reader = BufReader::new(File::open(tickers_file).unwrap());
        if let Err(quote_server) =
            QuoteServer::run_quote_server(&mut reader, server_addr, &udp_addr) {
            println!("Error: {}", quote_server);
        }
    }
}
