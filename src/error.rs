use std::fmt;
use std::io;

#[derive(Debug)]
pub enum GraiceError {
    Io(io::Error),
    ParseError(String),
    InvalidHeader(String),
    EmulationError(String),
    ScrollError(scroll::Error),
}

impl fmt::Display for GraiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraiceError::Io(err) => write!(f, "IO error: {}", err),
            GraiceError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            GraiceError::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
            GraiceError::EmulationError(msg) => write!(f, "Emulation error: {}", msg),
            GraiceError::ScrollError(err) => write!(f, "Scroll error: {}", err),
        }
    }
}

impl std::error::Error for GraiceError {}

impl From<io::Error> for GraiceError {
    fn from(err: io::Error) -> Self {
        GraiceError::Io(err)
    }
}

impl From<scroll::Error> for GraiceError {
    fn from(err: scroll::Error) -> Self {
        GraiceError::ScrollError(err)
    }
}

pub type Result<T> = std::result::Result<T, GraiceError>;
