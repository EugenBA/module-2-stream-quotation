use std::fmt::{Display, Formatter};
use std::io;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum QuoteGeneratorError
{
    BadParseQuote(String),
}
impl Display for QuoteGeneratorError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            QuoteGeneratorError::BadParseQuote(s) => write!(f, "File read error: {}", s),
        }
    }
}

impl  From<io::Error> for QuoteGeneratorError{
    fn from(err: io::Error) -> Self{
        QuoteGeneratorError::BadParseQuote(err.to_string())
    }
}