use clap::{Arg, Command};
use log::LevelFilter;
use std::path::Path;


pub(crate) struct CliArgs{
    pub server_addr: String,
    pub udp_addr: String,
    pub tickers_file: String,
    pub log_level: LevelFilter,
    pub file_log: String
}

impl CliArgs{
    pub fn get_cli_args() -> Option<Self> {
        let matches = Command::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .about(format!("Demo quote stream {}", env!("CARGO_PKG_NAME")))
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
            .arg(
                Arg::new("log-file")
                    .short('f')
                    .long("log-file")
                    .help("Log file")
                    .default_value("quote-client.log")
                    .required(false)
            )
            .get_matches();
        let server_addr = matches.get_one::<String>("server-addr");
        let udp_port = matches.get_one::<String>("udp-port");
        let tickers_file = matches.get_one::<String>("tickers-file");
        let log_level = matches.get_one::<String>("log-level");
        let log_file = matches.get_one::<String>("log-file");
        if let Some(server_addr) = server_addr &&
            let Some(udp_port) = udp_port && let Some(tickers_file) = tickers_file &&
            let Some(log_level) = log_level && let Some(log_file) = log_file{
            if !Path::new(tickers_file).exists() {
                eprintln!("File {} not exists", tickers_file);
                return None;
            }
            let level = {
                match log_level.as_ref() {
                    "DEBUG" => LevelFilter::Debug,
                    "ERROR" => LevelFilter::Error,
                    "WARN" => LevelFilter::Warn,
                    _ => LevelFilter::Info,
                }
            };
            let host_port: Vec<&str> = server_addr.split(":").collect();
            if host_port.len() < 2{
                eprintln!("Error address server");
            }
            let udp_addr = host_port[0].to_owned()  + ":" + udp_port;
            return Some(Self{
                server_addr: server_addr.to_owned(),
                udp_addr: udp_addr.to_owned(),
                tickers_file: tickers_file.to_string(),
                log_level: level,
                file_log: log_file.to_owned()
            })
        }
        None
    }
}
