//! Модуль релизующий тип хранения котировко
//!
//! Предосттавляет функциональность для чтения коировок из файла, сериализации и десереализаци

use std::io::Read;
use crate::errors::QuoteGeneratorError;
use chrono::{DateTime};

/// Структура для хранения данных котировок
#[derive(Debug, Clone, PartialEq)]
pub struct StockQuote {
    /// название котировки
    pub ticker: String,
    /// цена котировки
    pub price: f64,
    /// объем
    pub volume: u32,
    /// метка времени
    pub timestamp: u64,
}

impl StockQuote {
    /// Создает новую структуру StockQuote с заданным именнем котировки
    ///
    /// # Аргументы
    /// * `name_ticker` - Имя котировки
    ///
    /// # Возращает
    /// струткуру StockQuote
    pub fn new(name_ticker: &str) -> Self {
        StockQuote {
            ticker: name_ticker.to_string(),
            price: 0.0,
            volume: 0,
            timestamp: 0,
        }
    }

    /// ```rust
    /// Чтение  данных котировок в вектор as strings.
    ///
    /// # Параматеры
    /// * `R` -Тип реализующий трайт чтения.
    ///
    /// # Аргументы
    /// * `r` - Изменяемы тип для чтения данных
    ///          Входной формат название котировк разделенный символом переноса строки
    ///
    /// # Возращает
    /// * `Ok(Vec<String>)` - вектор имент котировок
    /// * `Err(QuoteGeneratorError)` - ошибку тип QuoteGeneratorError.
    ///
    /// # Ошибки
    /// Возращает ошибку в случае
    /// * Ошибку чтения с провайдера данных
    /// * Ошибку ввода-вывода
    ///
    /// # Пример
    /// ```rust
    /// use std::io::Cursor;
    /// use quote_lib::quote::stockquote::StockQuote;
    ///
    /// let data = "AAPL\nGOOG\nMSFT";
    /// let mut reader = Cursor::new(data);
    ///
    /// match StockQuote::get_tickers(&mut reader) {
    ///     Ok(tickers) => {
    ///         assert_eq!(tickers, vec!["AAPL".to_string(), "GOOG".to_string(), "MSFT".to_string()]);
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Error: {:?}", e);
    ///     }
    /// }
    /// ```
    /// ```
    pub fn get_tickers <R:Read> (r:&mut R) -> Result<Vec<String>, QuoteGeneratorError>{
        let mut quotes = String::new();
        r.read_to_string(&mut quotes)?;
        Ok(quotes.split("\n").map(|s| s.to_string()).collect())
    }

    /// ```rust
    /// use quote_lib::quote::stockquote::StockQuote;
    /// Преобразование тикеров из строки разделенных символом "," в структуру StockQuote
    ///
    /// # Аргументы
    ///
    /// * `tickers` - строка с именами тикеров с разделителм запята (e.g., "AAPL,MSFT,GOOG").
    ///
    /// # Возращает
    ///
    /// Возращает `Vec<StockQuote>`
    ///
    /// # Пример
    ///
    /// ```
    /// let tickers = "AAPL,MSFT,GOOG";
    /// let stock_quotes = StockQuote::get_tickers_subscribe(tickers);
    /// assert_eq!(stock_quotes.len(), 3);
    /// ```
    /// ```
    pub fn get_tickers_subscribe(tickers: &str) -> Vec<StockQuote>{
        let tickers : Vec<StockQuote> = tickers.split(",").map(|s| 
            StockQuote::new(s)).collect();
        tickers
    }

    /// ```rust
    ///
    ///  Читает данные котировок из потока (реализующий трейт read) и возращает строк с именами котировк разделенных запятой
    ///
    ///  # Тип параметра
    ///  * - `R`: тип реализующий трейт Read
    ///
    ///  # Параметр
    ///  * - `r`: мутабельная ссылка на тип реализующий трейт Read
    ///
    ///  # Возращает
    ///  * - `Ok(String)`: имена котировк разделенных запаятой
    ///  * - `Err(QuoteGeneratorError)`: ошибку при парсинге данных
    ///
    ///  # Errors
    ///  * Ошибки чтения
    ///  * Ошибки парсинга
    ///
    ///  # Пример
    ///  ```
    /// use std::io::Cursor;
    /// use quote_lib::quote::stockquote::StockQuote;
    ///
    /// let data = "AAPL\nMSFT\nGOOGL\n";
    /// let mut cursor = Cursor::new(data);
    ///
    /// let result = StockQuote::get_tickers_string_from_file(&mut cursor);
    /// assert_eq!(result.unwrap(), "AAPL,MSFT,GOOGL");
    ///   ```
    ///
    /// ```
    pub fn get_tickers_string_from_file<R:Read>(r:&mut R) -> Result<String, QuoteGeneratorError>{
        let vec = Self::get_tickers(r)?;
        Ok(vec.join(","))
    }
   /// ```rust
   /// use quote_lib::quote::stockquote::StockQuote;
   /// Конвертация структуры StockQuote в строку с разделителем "|"
   ///
   /// Метод сереализует структуру в строку с разделителями
   ///
   ///
   /// # Возращает
   ///
   /// Строку с разделителями формата:
   /// `"<ticker>|<price>|<volume>|<timestamp>"`.
   ///
   /// # Пример
   ///
   /// ```
   /// let stock = StockQuote {
   ///     ticker: String::from("AAPL"),
   ///     price: 150.23,
   ///     volume: 1200,
   ///     timestamp: 1633045692,
   /// };
   /// let serialized = stock.to_string();
   /// assert_eq!(serialized, "AAPL|150.23|1200|1633045692");
   /// ```
   /// ```
   pub fn to_string(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.ticker, self.price, self.volume, self.timestamp
        )
    }

    /// ```rust
    /// use quote_lib::quote::stockquote::StockQuote;
    /// use chrono::{DateTime};
    ///
    /// Конвертация структуры StockQuote в формат json
    ///
    /// Метод сереализует структуру в строку с разделителями
    ///
    ///
    /// # Возращает
    ///
    /// Строку с разделителями формата:
    /// `"{"ticker": "", "price": "volume": "timestamp": ""`.
    ///
    /// # Пример
    ///
    /// ```
    /// let stock = StockQuote {
    ///     ticker: String::from("AAPL"),
    ///     price: 150.23,
    ///     volume: 1200,
    ///     timestamp: 1633045692,
    /// };
    /// let serialized = stock.to_json();
    /// assert_eq!(serialized, "{\"ticker\": \"AAPL\",  \"price\": 150.23, \"volume\": 1200, \"timestamp\": \"1970-01-19T21:37:25\"}");
    /// ```
    /// ```

    pub fn to_json(&self) -> Result<String, QuoteGeneratorError> {
        let date_time =  DateTime::from_timestamp_millis(self.timestamp as i64);
        if let Some(date_time) = date_time {
            return Ok(format!(
                "{{\"ticker\": \"{}\", \"price\": {}, \"volume\": {}, \"timestamp\": \"{}\"}}",
                self.ticker, self.price,
                self.volume,
                date_time.format("%Y-%m-%dT%H:%M:%S")
            ));
        }
        Err(QuoteGeneratorError::BadParseTimestampQuote("Error parse".to_string()))
    }

    /// ```rust
    /// Создает экземпляр струтуры StockQuote из строки с разделителем "|"
    ///
    ///
    /// # Параметр
    /// - `s`: строковой тип с формата `"<ticker>|<price>|<volume>|<timestamp>"`.
    ///
    /// # Возращает
    /// - `Some(StockQuote)`если десериализация прошла без ошибок
    /// - `None` если формат входных данных не соовествует
    ///
    /// # Пример
    ///
    /// ```rust
    /// use quote_lib::quote::stockquote::StockQuote;
    ///
    /// let input = "AAPL|157.92|300000|1697071010";
    /// if let Some(stock) = StockQuote::from_string(input) {
    ///     println!("Parsed stock quote: {:?}", stock);
    /// } else {
    ///     println!("Failed to parse stock quote.");
    /// }
    /// ```
    ///
    /// ```rust
    /// use quote_lib::quote::stockquote::StockQuote;
    ///
    /// let invalid_input = "AAPL|157.92|INVALID_VOLUME|1697071010";
    /// assert!(StockQuote::from_string(invalid_input).is_none());
    /// ```
    /// ```
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

    /// ```rust
    /// Сереализует структуру StockQuote в байтовый вектор (`Vec<u8>`).
    ///
    ///
    /// # Возращает:
    /// `Vec<u8>` вектор байт
    ///
    /// # Пример:
    /// ```rust
    /// use quote_lib::quote::stockquote::StockQuote;
    ///
    /// let data = StockQuote {
    ///     ticker: "AAPL".to_string(),
    ///     price: 150.34,
    ///     volume: 2000,
    ///     timestamp: 1672531200,
    /// };
    /// let serialized = data.to_bytes();
    /// assert_eq!(String::from_utf8(serialized).unwrap(), "AAPL|150.34|2000|1672531200\n");
    /// ```
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


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new(){
        let test_quote = StockQuote{
            ticker: "test".to_string(),
            price: 0.0,
            volume: 0,
            timestamp: 0,
        };
        let quote = StockQuote::new("test");
        assert_eq!(test_quote, quote);
    }

    #[test]
    fn test_get_tickers(){
        let test_str = "A,B";
        let test_vec : Vec<StockQuote> = vec![StockQuote::new("A"), StockQuote::new("B")];
        let quote_vec = StockQuote::get_tickers_subscribe(test_str);
        assert_eq!(test_vec, quote_vec);
    }

    #[test]
    fn test_to_string(){
        let test_str = "test|0.0|0|0".to_string();
        let test_quote = StockQuote::new("test");
        let quote = StockQuote::from_string(&test_str).unwrap();
        assert_eq!(test_quote, quote);
    }

    #[test]
    fn test_to_json(){
        let stock = StockQuote {
             ticker: String::from("AAPL"),
             price: 150.23,
             volume: 1200,
             timestamp: 1633045692,
         };
        let serialized = stock.to_json().unwrap();
        assert_eq!(serialized, "{\"ticker\": \"AAPL\",  \"price\": 150.23, \"volume\": 1200, \"timestamp\": \"1970-01-19T21:37:25\"}");
    }

    #[test]
    fn test_from_string(){
        let test_str = "AAPL|157.92|300000|1697071010".to_string();
        let test_quote = StockQuote{ticker: "AAPL".to_string(), price: 157.92, volume: 300000,
        timestamp: 1697071010};
        let quote = StockQuote::from_string(&test_str).unwrap();
        assert_eq!(test_quote, quote);
    }

    #[test]
    fn test_to_bytes(){
        let test_vec: Vec<u8> = vec![116, 101, 115, 116, 124, 48, 124, 48, 124, 48, 10];
        let quote = StockQuote::new("test");
        assert_eq!(test_vec, quote.to_bytes());
    }
}
