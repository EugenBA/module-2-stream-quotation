use std::net::UdpSocket;
use crate::error::QuoteStreamServerError;

pub(crate) struct QuoteStream {
    socket: UdpSocket
}

impl QuoteStream {
    pub fn new(bind_adr: &str) -> Result<Self, QuoteStreamServerError> {
        Ok(Self {
            socket: UdpSocket::bind(bind_adr)?
        })
    }

    pub fn send_quote(&self, quote: &str) -> Result<(), QuoteStreamServerError>{
        self.socket.send(quote.as_bytes())?;
        Ok(())
    }

}