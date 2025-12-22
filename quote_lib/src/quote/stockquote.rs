use std::io::Read;
use crate::errors::QuoteGeneratorError;

#[derive(Debug, Clone)]
pub struct StockQuote {
    pub ticker: String,
    pub price: f64,
    pub volume: u32,
    pub timestamp: u64,
}

impl StockQuote {
    pub fn new(name_ticker: &str) -> Self {
        StockQuote {
            ticker: name_ticker.to_string(),
            price: -9999.0,
            volume: 0,
            timestamp: 0,
        }
    }
    pub fn get_tickers <R:Read> (r:&mut R) -> Result<Vec<String>, QuoteGeneratorError>{
        let mut quotes = String::new();
        r.read_to_string(&mut quotes)?;
        Ok(quotes.split("\n").map(|s| s.to_string()).collect())
    }
    
    pub fn get_tickers_subscribe(tickers: &str) -> Vec<StockQuote>{
        let tickers : Vec<StockQuote> = tickers.split(",").map(|s| 
            StockQuote::new(s)).collect();
        tickers
    }
    
    pub fn get_tickers_string_from_file<R:Read>(r:&mut R) -> Result<String, QuoteGeneratorError>{
        let vec = Self::get_tickers(r)?;
        Ok(vec.join(","))
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.ticker, self.price, self.volume, self.timestamp
        )
    }

    pub fn from_string(s: &str) -> Option<Self> {
        let binding = s.replace('\n', "");
        let parts: Vec<&str> = binding.split('|').collect();
        if parts.len() == 4 {
            Some(StockQuote {
                ticker: parts[0].to_string(),
                price: parts[1].parse().ok()?,
                volume: parts[2].parse().ok()?,
                timestamp: parts[3].parse().ok()?,
                
            })
        } else {
            None
        }
    }

    // Или бинарная сериализация
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.ticker.as_bytes());
        bytes.push(b'|');
        bytes.extend_from_slice(self.price.to_string().as_bytes());
        bytes.push(b'|');
        bytes.extend_from_slice(self.volume.to_string().as_bytes());
        bytes.push(b'|');
        bytes.extend_from_slice(self.timestamp.to_string().as_bytes());
        bytes.push(b'\n');
        bytes
    }
}