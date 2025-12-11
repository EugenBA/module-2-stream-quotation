use std::fmt::{Display, Formatter};
use std::io;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum QuoteStreamServerError
{
    BadNetworkBindSocket(String),
}
impl Display for QuoteStreamServerError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            QuoteStreamServerError::BadNetworkBindSocket(s) => write!(f, "File read error: {}", s),
        }
    }
}

impl  From<io::Error> for QuoteStreamServerError{
    fn from(err: io::Error) -> Self{
        QuoteStreamServerError::BadNetworkBindSocket(err.to_string())
    }
}