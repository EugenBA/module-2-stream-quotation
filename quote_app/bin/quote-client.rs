
    #![warn(missing_docs)]
    //! Приложение - клиент для получения стриминнга котировк
    //!
    //! Производит запрос к среверу и по UDP получает поток данных по котировкам.
    #![allow(unused_imports, unused_variables)]
    #[path = "../src/client.rs"]
    mod client;
    #[path = "../src/error.rs"]
    mod error;
    #[path = "../src/parsecli.rs"]
    mod parsecli;
    #[path = "../src/logger.rs"]
    mod logger;


    use std::fs::File;
    use std::io::BufReader;
    use quote_lib::quote::stockquote::StockQuote;
    #[cfg(feature = "client")]
    use crate::client::QuoteStreamClient;
    use log::{warn};
    use crate::logger::setup_logger;
    use crate::parsecli::CliArgs;


    fn main() {
        #[cfg(feature = "client")]{
            let cli_args = CliArgs::get_cli_args();
            if let Some(arg) = cli_args {
                setup_logger(arg.log_level, &arg.file_log);
                let mut reader = BufReader::new(File::open(arg.tickers_file).unwrap());
                let tickers = StockQuote::get_tickers_string_from_file(&mut reader).unwrap();
                let mut quote_stream_client = QuoteStreamClient::default();
                if let Err(e) = quote_stream_client.get_quote_stream(&arg.udp_addr,
                                                                     &arg.server_addr, tickers)
                {
                    println!("Error: {}", e);
                }
            } else {
                print!("Bad command args")
            }
        }
    }

