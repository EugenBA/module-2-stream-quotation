use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum QuoteClientError
{
    #[error("Bad network bind socket: {0}")]
    BadNetworkBindSocket(String),
    #[error("Bad network create Tcp Socket connect: {0}")]
    BadTcpSocketConnect(String),
}

impl  From<io::Error> for QuoteClientError{
fn from(err: io::Error) -> Self{
    QuoteClientError::BadNetworkBindSocket(err.to_string())
}
}