use std::{io, net};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum QuoteClientError
{
    #[error("Bad network bind socket: {0}")]
    BadNetworkBindSocket(String),
    #[error("Error parse network address: {0}")]
    AddressParseError(String),
}

impl  From<io::Error> for QuoteClientError{
fn from(err: io::Error) -> Self{
    QuoteClientError::BadNetworkBindSocket(err.to_string())
}
}

impl From<net::AddrParseError>  for QuoteClientError{
    fn from(err: net::AddrParseError) -> Self {
       QuoteClientError::AddressParseError(err.to_string())
    }
}