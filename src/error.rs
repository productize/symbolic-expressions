// (c) 2016 Productize SPRL <joost@productize.be>

use std::string::FromUtf8Error;
use std::error;
use std::io;
use std::fmt;
use std::result;
use serde::de;

/// Error type for symbolic-expressions
#[derive(Debug)]
pub enum Error {
    /// any other error type
    Other(String),
    /// intended to give more detailed parser errrors, actually not used currently
    Parse(ParseError, String),
    /// IO error
    Io(io::Error),
    /// UTF8 parsing error
    FromUtf8(FromUtf8Error),
    /// decoder error
    Decoder(String),
}

/// detailed symbolic-expression parse error information
#[derive(Debug)]
pub struct ParseError {
    msg: String,
    line: usize,
    col: usize,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Other(ref s) => s,
            Error::Decoder(ref s) => s,
            Error::Parse(_, ref pe) => pe,
            Error::Io(ref error) => error::Error::description(error),
            Error::FromUtf8(ref error) => error.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref error) => Some(error),
            Error::FromUtf8(ref error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Error {
        Error::FromUtf8(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match *self {
            Error::Other(ref s) => write!(fmt, "Error:{}", s),
            Error::Decoder(ref s) => write!(fmt, "Decoder Error:{}", s),
            Error::Parse(ref pe, _) => write!(fmt, "Parse Error {}:{} {}", pe.line, pe.col, pe.msg),
            Error::Io(ref error) => fmt::Display::fmt(error, fmt),
            Error::FromUtf8(ref error) => fmt::Display::fmt(error, fmt),
        }
    }
}

impl de::Error for Error {
    fn custom<T: Into<String>>(msg: T) -> Self {
        Error::Decoder(msg.into())
    }

    fn end_of_stream() -> Self {
        Error::Decoder("end_of_stream".into())
    }
}

/// symbolic-expressions Result type
pub type Result<T> = result::Result<T, Error>;

/// utility function that creates a symbolic-expressions Error Result from a String
pub fn str_error<T>(s: String) -> Result<T> {
    Err(Error::Other(s))
}

/// utility function that creates a symbolic-expressions Error Result for a parser error
pub fn parse_error<T>(line: usize, col: usize, msg: String) -> Result<T> {
    let pe = ParseError {
        msg: msg,
        line: line,
        col: col,
    };
    let s = format!("Parse Error: {}:{} {}", line, col, pe.msg);
    Err(Error::Parse(pe, s))
}
