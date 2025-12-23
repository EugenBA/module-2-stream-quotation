use std::io;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum QuoteGeneratorError
{
    #[error("Bad parse quote: {0}")]
    BadParseQuote(String),

}

impl  From<io::Error> for QuoteGeneratorError{
    fn from(err: io::Error) -> Self{
        QuoteGeneratorError::BadParseQuote(err.to_string())
    }
}

