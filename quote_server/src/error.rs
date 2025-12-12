use std::fmt::{Display};
use std::io;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum QuoteStreamServerError
{
    #[error("Bad network bind socket: {0}")]
    BadNetworkBindSocket(String),
    #[error("KeepAlive timeout error: {0}")]
    KeepAliveTimeoutError(String),
    #[error("Generate quote error: {0}")]
    GeneratorQuoteError(String)
}

impl  From<io::Error> for QuoteStreamServerError{
    fn from(err: io::Error) -> Self{
        QuoteStreamServerError::BadNetworkBindSocket(err.to_string())
    }
}