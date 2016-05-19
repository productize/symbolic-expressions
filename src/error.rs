// (c) 2016 Productize SPRL <joost@productize.be>

use std::string::FromUtf8Error;
use std::error;
use std::io;
use std::fmt;
use std::result;

#[derive(Debug)]
pub enum Error {
    Other(String),
    Parse(ParseError, String),
    Io(io::Error),
    FromUtf8(FromUtf8Error),
}

// TODO: actually use ParseError
// looks like nom does not give a good way to be able to see line:col
#[derive(Debug)]
pub struct ParseError {
    msg:String,
    line:i64,
    col:i64,
}

impl error::Error for Error {
    
    fn description(&self) -> &str {
        match *self {
            Error::Other(ref s) => s,
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
            Error::Other(ref s) => {
                write!(fmt, "Error:{}", s)
            }
            Error::Parse(ref pe, _) => {
                write!(fmt, "Parse Error {}:{} {}", pe.line, pe.col, pe.msg)
            }
            Error::Io(ref error) => fmt::Display::fmt(error, fmt),
            Error::FromUtf8(ref error) => fmt::Display::fmt(error, fmt),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

pub fn str_error<T>(s:String) -> Result<T> {
    Err(Error::Other(s))
}

pub fn parse_error<T>(line:i64, col:i64, msg:String) -> Result<T> {
    let pe = ParseError {
        msg:msg,
        line:line,
        col:col,
        
    };
    let s = format!("Parse Error: {}:{} {}", line, col, pe.msg);
    Err(Error::Parse(pe, s))
}
