//! Модуль релизующий тип хранения котировко
//!
//! Предосттавляет функциональность для чтения коировок из файла, сериализации и десереализаци

use std::io::Read;
use crate::errors::QuoteGeneratorError;

/// Структура для хранения данных котировок
#[derive(Debug, Clone)]
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
    ///
    /// let data = "AAPL\nGOOG\nMSFT";
    /// let mut reader = Cursor::new(data);
    ///
    /// match get_tickers(&mut reader) {
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
    /// let stock_quotes = get_tickers_subscribe(tickers);
    /// assert_eq!(stock_quotes.len(), 3); // Converts and collects into a vector of `StockQuote`
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
    ///  use std::io::Cursor;
    ///
    ///  let data = "AAPL\nMSFT\nGOOGL\n";
    ///  let mut cursor = Cursor::new(data);
    ///
    ///  let result = YourStruct::get_tickers_string_from_file(&mut cursor);
    ///  assert_eq!(result.unwrap(), "AAPL,MSFT,GOOGL");
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
   /// let stock = Stock {
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
    /// let input = "AAPL|157.92|300000|1697071010";
    /// if let Some(stock) = StockQuote::from_string(input) {
    ///     println!("Parsed stock quote: {:?}", stock);
    /// } else {
    ///     println!("Failed to parse stock quote.");
    /// }
    /// ```
    ///
    /// ```rust
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
    /// Сереализует структуру StockQuote в батовый вектор (`Vec<u8>`).
    ///
    ///
    /// # Возращает:
    /// `Vec<u8>` вектор байт
    ///
    /// # Пример:
    /// ```rust
    /// let data = MyStruct {
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