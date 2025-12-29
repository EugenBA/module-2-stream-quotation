//! Модуль для реализации обработки ошибок
//!
//! Предоставляет функциональность по обработке ошибок генератора котировок

use std::io;
use thiserror::Error;
/// Перчесление для ошибок генератора котировок
/// !
/// Представляет функциональность по обработке ошибок
#[derive(Error, Debug)]
pub enum QuoteGeneratorError
{
    /// ошибка парсинга котировк
    #[error("Bad parse quote: {0}")]
    BadParseQuote(String),
    #[error("Bad parse timestamp: {0}")]
    /// ошибка парсинга даты времени из таймстампа
    BadParseTimestampQuote(String)

}

impl  From<io::Error> for QuoteGeneratorError{
    fn from(err: io::Error) -> Self{
        QuoteGeneratorError::BadParseQuote(err.to_string())
    }
}

