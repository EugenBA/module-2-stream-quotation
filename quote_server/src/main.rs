#![warn(missing_docs)]
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
        .filter(None, LevelFilter::Debug) // Уровень по умолчанию
        .target(Target::Stdout) // Вывод в stdout вместо stderr
        .write_style(env_logger::WriteStyle::Always) // Всегда использовать цвета
        .init();
}

fn main() {
    /*let matches = Command::new("quote-server")
        .version("0.1.0")
        .about("Demo quote stream server")
        .arg(
            Arg::new("local")
                .short('l')
                .long("local")
                .help("Local address: host:port")
                .required(true)
        )
        .arg(
            Arg::new("udp")
                .short('u')
                .long("udp")
                .help("Server udp bind address: host:port")
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
    let local = matches.get_one::<String>("local");
    let udp = matches.get_one::<String>("udp");
    let tickets_file = matches.get_one::<String>("tickets");*/
    setup_logger();
    let local = Some("127.0.0.1:8120");
    let udp = Some("127.0.0.1:55505");
    let tickets_file = Some("/home/eugen/RustroverProjects/module-2-stream-quotation/tickets/tickets.txt");
    if let Some(local) = local && let Some(udp) = udp && let Some(tickets_file) = tickets_file {
        if !Path::new(&tickets_file).exists() {
            eprintln!("Файл {} не существует", tickets_file);
            return;
        }
        let mut reader = BufReader::new(File::open(tickets_file).unwrap());
        if let Err(quote_server) =
            QuoteServer::run_quote_server(&mut reader, local, udp) {
            println!("Error: {}", quote_server);
        }
    }
}
