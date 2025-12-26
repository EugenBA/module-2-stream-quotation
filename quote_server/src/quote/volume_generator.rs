use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crossbeam_channel::{Sender};
use quote_lib::quote::stockquote::{StockQuote};
use crate::error::QuoteStreamServerError;

//константа для генрации стоимости дорогих котировок
const BIG_PRICE:f64 = 210.0;
//констатнта генрации котировк для "дешевых" котировк
const LOW_PRICE:f64 = 40.0;
//пауза потока обновления котировок
const WAIT_MILLISECOND_NEXT_GENERATION: u64 = 100;


#[derive(Default)]
pub(crate) struct QuoteGenerator{
}


impl QuoteGenerator {

    fn generate_quote(ticker: &str) -> Option<StockQuote> {
        //генрация стоимости котировк
        let last_price = match ticker{
            // Популярные акции имеют больший объём
                "AAPL" | "MSFT" | "TSLA" => BIG_PRICE + (rand::random::<f64>() * BIG_PRICE*0.05),
            // Обычные акции - средний объём
            _ => LOW_PRICE + (rand::random::<f64>() * LOW_PRICE*0.9),
        };
        //генерация объемов котировок
        let volume = match ticker {
            // Популярные акции имеют больший объём
            "AAPL" | "MSFT" | "TSLA" => 1000 + (rand::random::<f64>() * 5000.0) as u32,
            // Обычные акции - средний объём
            _ => 100 + (rand::random::<f64>() * 1000.0) as u32,
        };

        Some(StockQuote {
            ticker: ticker.to_string(),
            price: last_price,
            volume,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH).ok()?
                .as_millis() as u64,
        })
    }
    pub(crate) fn thread_generate(s: Sender<StockQuote>, tickers: &Vec<String>) -> Result<(), QuoteStreamServerError>{
        loop {
            for ticker in tickers {
               if let Some(quote) = QuoteGenerator::generate_quote(ticker){
                   if let Err(e ) = s.send(quote){
                       return Err(QuoteStreamServerError::GeneratorQuoteError(format!("Error sender quote {}", e)));
                   }
               }
            }
            thread::sleep(Duration::from_millis(WAIT_MILLISECOND_NEXT_GENERATION));
        }
    }
}
