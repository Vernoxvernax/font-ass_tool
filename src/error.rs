use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    Message(String),
    MissingScriptInfo,
    MissingStylesInfo,
    MissingEvents,
    Syntax,
    FailedParsingFonts
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::MissingEvents => formatter.write_str("Missing events"),
            Error::MissingScriptInfo => formatter.write_str("Missing script info"),
            Error::MissingStylesInfo => formatter.write_str("Missing styles info"),
            Error::Syntax => formatter.write_str("Invalid syntax"),
            Error::FailedParsingFonts => formatter.write_str("Failed to parse font names")
        }
    }
}

impl std::error::Error for Error {}
