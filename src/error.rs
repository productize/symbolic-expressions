// (c) 2016-2017 Productize SPRL <joost@productize.be>

use std::string;
use std::io;
use std::num;

/// errors that can happen in this library
#[derive(Debug)]
pub enum SexpError {
    /// parse error
    Parse(ParseError),
    /// other error
    Other(String),
    /// IO Error
    Io(io::Error),
    /// Utf8 Error parsing error
    FromUtf8(string::FromUtf8Error),
    /// floating point parsing error
    Float(num::ParseFloatError),
    /// integer parsing error
    Int(num::ParseIntError),
}

pub use SexpError as Error;

/// detailed symbolic-expression parse error information
#[derive(Debug)]
pub struct ParseError {
    msg: String,
    line: usize,
    col: usize,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<String> for Error {
    fn from(e: String) -> Error {
        Error::Other(e)
    }
}

impl<'a> From<&'a str> for Error {
    fn from(e: &'a str) -> Error {
        Error::Other(e.into())
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(e: string::FromUtf8Error) -> Error {
        Error::FromUtf8(e)
    }
}

impl From<num::ParseFloatError> for Error {
    fn from(e: num::ParseFloatError) -> Error {
        Error::Float(e)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(e: num::ParseIntError) -> Error {
        Error::Int(e)
    }
}

/// utility function that creates a symbolic-expressions Error Result for a parser error
pub fn parse_error<T>(line: usize, col: usize, msg: String) -> Result<T, Error> {
    let pe = ParseError {
        msg: msg,
        line: line,
        col: col,
    };
    Err(Error::Parse(pe))
}
