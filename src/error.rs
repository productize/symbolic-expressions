use std::string::FromUtf8Error;
use std::error;
use std::io;
use std::fmt;
use std::result;

#[derive(Debug)]
pub enum Error {
    Other(String), // TODO: line/column error for parser
    Io(io::Error),
    FromUtf8(FromUtf8Error),
}


impl error::Error for Error {
    
    fn description(&self) -> &str {
        match *self {
            Error::Other(ref s) => s,
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
            Error::Io(ref error) => fmt::Display::fmt(error, fmt),
            Error::FromUtf8(ref error) => fmt::Display::fmt(error, fmt),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

pub fn str_error<T>(s:String) -> Result<T> {
    Err(Error::Other(s))
}
