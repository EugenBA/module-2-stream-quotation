use std::io;
use std::time::SystemTimeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum QuoteStreamServerError
{
    #[error("Bad network bind socket: {0}")]
    BadNetworkBindSocket(String),
    #[error("Bad network create TcpStream: {0}")]
    BadCreateTcpStream(String),
    #[error("Generate quote error: {0}")]
    GeneratorQuoteError(String),
    #[error("Bad set system time: {0}")]
    BadSetSystemTimeError(String),
    #[error("State change error: {0}")]
    ChangeThreadStateError(String)
}

impl  From<io::Error> for QuoteStreamServerError{
    fn from(err: io::Error) -> Self{
        QuoteStreamServerError::BadNetworkBindSocket(err.to_string())
    }
}

impl  From<SystemTimeError> for QuoteStreamServerError{
    fn from(err: SystemTimeError) -> Self{
        QuoteStreamServerError::BadSetSystemTimeError(err.to_string())
    }
}