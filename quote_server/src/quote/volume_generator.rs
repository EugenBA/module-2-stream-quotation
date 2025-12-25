use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crossbeam_channel::{Sender};
use quote_lib::quote::stockquote::{StockQuote};
use crate::error::QuoteStreamServerError;

const BIG_PRICE:f64 = 210.0;
const LOW_PRICE:f64 = 40.0;
const WAIT_MILLISECOND_NEXT_GENERATION: u64 = 100;


#[derive(Default)]
pub(crate) struct QuoteGenerator{
}


impl QuoteGenerator {

    fn generate_quote(ticker: &str) -> Option<StockQuote> {

        let last_price = match ticker{
            // Популярные акции имеют больший объём
                "AAPL" | "MSFT" | "TSLA" => BIG_PRICE + (rand::random::<f64>() * BIG_PRICE*0.05),
            // Обычные акции - средний объём
            _ => LOW_PRICE + (rand::random::<f64>() * LOW_PRICE*0.9),
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
                .duration_since(UNIX_EPOCH).ok()?
                .as_millis() as u64,
        })
    }
    /// ```
    /// Spawns a thread to generate stock quotes for a given list of tickers and send them to a channel.
    ///
    /// # Arguments
    ///
    /// * `s` - A `Sender` object from the `std::sync::mpsc` module used to transmit generated stock quotes.
    /// * `tickers` - A vector of strings representing the stock ticker symbols for which quotes are to be generated.
    ///
    /// # Returns
    ///
    /// * `Result<(), QuoteStreamServerError>` - Returns `Ok(())` if the operation successfully runs without errors.
    ///   Returns an error of type `QuoteStreamServerError` if sending a quote through the channel fails.
    ///
    /// # Behavior
    ///
    /// * The function enters an infinite loop that iterates through the provided list of `tickers`.
    /// * For each `ticker`, it calls the `QuoteGenerator::generate_quote(ticker)` function to generate a stock quote.
    /// * If a quote is successfully generated, it attempts to send it using the provided `Sender`.
    ///   If sending fails, the function terminates early and returns an error containing the failure details.
    /// * After processing all tickers, the thread sleeps for a predefined duration (`WAIT_MILLISECOND_NEXT_GENERATION`)
    ///   before repeating the process.
    ///
    /// # Errors
    ///
    /// * If sending a stock quote via the `Sender` fails, the function returns a `QuoteStreamServerError::GeneratorQuoteError`
    ///   with a detailed error message.
    ///
    /// # Note
    ///
    /// This function runs indefinitely in its current form, producing stock quotes and transmitting them in a loop.
    /// Ensure to run it in a separate thread or context where such behavior is acceptable.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::sync::mpsc::channel;
    /// use std::thread;
    ///
    /// let tickers = vec![String::from("AAPL"), String::from("GOOG"), String::from("AMZN")];
    /// let (tx, rx) = channel();
    ///
    /// thread::spawn(move || {
    ///     thread_generate(tx, &tickers).expect("Failed to generate quotes");
    /// });
    ///
    /// // Receiver can now process stock quotes from the channel.
    /// ```
    /// ```
    pub fn thread_generate(s: Sender<StockQuote>, tickers: &Vec<String>) -> Result<(), QuoteStreamServerError>{
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
