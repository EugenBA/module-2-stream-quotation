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
    /// Converts a comma-separated string of stock ticker symbols into a vector of `StockQuote` objects.
    ///
    /// # Arguments
    ///
    /// * `tickers` - A string slice containing comma-separated stock ticker symbols (e.g., "AAPL,MSFT,GOOG").
    ///
    /// # Returns
    ///
    /// A `Vec<StockQuote>` where each element is created by invoking the `StockQuote::new`
    /// function on the individual ticker symbols extracted from the input string.
    ///
    /// # Example
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
    ///     /**
    ///      * Reads ticker symbols from a given readable source, parses them into a vector,
    ///      * and returns a comma-separated string of the tickers.
    ///      *
    ///      * # Type Parameters
    ///      * - `R`: A type that implements the `Read` trait, representing the readable input source.
    ///      *
    ///      * # Parameters
    ///      * - `r`: A mutable reference to the input source implementing `Read`. This serves
    ///      *        as the source from which ticker data will be read.
    ///      *
    ///      * # Returns
    ///      * - `Ok(String)`: A comma-separated string of ticker symbols if parsing and
    ///      *                 reading are successful.
    ///      * - `Err(QuoteGeneratorError)`: An error if reading the input or parsing the tickers fails.
    ///      *
    ///      * # Errors
    ///      * This function will return a `QuoteGeneratorError` if:
    ///      * - The input source cannot be read.
    ///      * - The ticker data cannot be parsed correctly.
    ///      *
    ///      * # Dependencies
    ///      * This function depends on `Self::get_tickers`, which extracts and returns
    ///      * the tickers in a `Vec<String>`.
    ///      *
    ///      * # Example
    ///      * ```
    ///      * use std::io::Cursor;
    ///      *
    ///      * let data = "AAPL\nMSFT\nGOOGL\n";
    ///      * let mut cursor = Cursor::new(data);
    ///      *
    ///      * let result = YourStruct::get_tickers_string_from_file(&mut cursor);
    ///      * assert_eq!(result.unwrap(), "AAPL,MSFT,GOOGL");
    ///      * ```
    ///      */
    /// ```
    pub fn get_tickers_string_from_file<R:Read>(r:&mut R) -> Result<String, QuoteGeneratorError>{
        let vec = Self::get_tickers(r)?;
        Ok(vec.join(","))
    }
   /// ```rust
   /// Converts the fields of the struct into a formatted `String`.
   ///
   /// This method serializes the values of the `ticker`, `price`, `volume`,
   /// and `timestamp` fields of the struct into a pipe (`|`)-separated string format.
   ///
   /// # Returns
   ///
   /// A `String` containing the serialized representation of the struct's fields
   /// in the format `"<ticker>|<price>|<volume>|<timestamp>"`.
   ///
   /// # Example
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
    /// Constructs an instance of `StockQuote` from a formatted string.
    ///
    /// The input string is expected to contain exactly 4 components separated by the `|` character,
    /// representing the `ticker`, `price`, `volume`, and `timestamp` fields of a `StockQuote`,
    /// in that order. The method will also remove any newline characters (`\n`) from the input
    /// string before parsing.
    ///
    /// # Parameters
    /// - `s`: A reference to a string slice containing the data to be parsed into a `StockQuote`.
    ///
    /// # Returns
    /// - `Some(StockQuote)` if the input string has the correct format and all components can be
    ///   successfully parsed:
    ///   - `ticker` is taken as-is from the first component.
    ///   - `price` is parsed as a `f64` from the second component.
    ///   - `volume` is parsed as a `u32` from the third component.
    ///   - `timestamp` is parsed as a `u64` from the fourth component.
    /// - `None` if the input string has fewer than 4 components or if any of the components fail
    ///   to parse into their expected types.
    ///
    /// # Examples
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
    /// Converts the current instance of the object into a vector of bytes (`Vec<u8>`).
    ///
    /// # Structure of the Output:
    /// The serialized byte vector represents the object's properties concatenated
    /// with specific delimiters in the following format:
    ///
    /// `ticker|price|volume|timestamp\n`
    ///
    /// - `ticker`: The string representation of the ticker (e.g., "AAPL").
    /// - `price`: The string representation of the price (e.g., "150.34").
    /// - `volume`: The string representation of the volume (e.g., "2000").
    /// - `timestamp`: The string representation of the timestamp (e.g., "1672531200").
    /// - Fields are separated by a `|` delimiter.
    /// - The serialized data ends with a newline character (`\n`).
    ///
    /// # Returns:
    /// A `Vec<u8>` containing the serialized representation of the object.
    ///
    /// # Example:
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
    ///
    /// # Notes:
    /// - It's expected that the fields `ticker`, `price`, `volume`, and `timestamp`
    ///   are accessible and can be converted into their byte-string representation.
    /// - Proper error handling or validation of the object's fields is not implemented
    ///   in this function and must be ensured prior to calling it.
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