use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use quote_lib::quote::stockquote::{StockQuote};

const BIG_PRICE:f64 = 210.0;
const LOW_PRICE:f64 = 120.0;

struct QuoteGenerator{
    quotes: Arc<RwLock<Vec<StockQuote>>>
}


impl QuoteGenerator {

    pub fn generate_quote(&mut self, ticker: &str) -> Option<StockQuote> {

        let last_price = match ticker{
            // Популярные акции имеют больший объём
                "AAPL" | "MSFT" | "TSLA" => BIG_PRICE + (rand::random::<f64>() * BIG_PRICE*0.05),
            // Обычные акции - средний объём
            _ => LOW_PRICE + (rand::random::<f64>() * LOW_PRICE*0.1),
        };
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
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        })
    }
}
