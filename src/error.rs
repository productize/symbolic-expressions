// (c) 2016-2017 Productize SPRL <joost@productize.be>

use std::string;
use std::io;
use std::num;

error_chain! {

    errors {
        /// parser error
        Parse(t: ParseError, s: String) {
            description("parse error")
            display("parse error: '{}'", s)
        }
    }
    
    foreign_links {
        Io(io::Error) #[doc = "IO error"];
        FromUtf8(string::FromUtf8Error) #[doc = "Utf8 error"];
        Float(num::ParseFloatError) #[doc = "Float error"];
        Int(num::ParseIntError) #[doc = "Int error"];
    }
}

/// detailed symbolic-expression parse error information
#[derive(Debug)]
pub struct ParseError {
    msg: String,
    line: usize,
    col: usize,
}

/// utility function that creates a symbolic-expressions Error Result for a parser error
pub fn parse_error<T>(line: usize, col: usize, msg: String) -> Result<T> {
    let pe = ParseError {
        msg: msg,
        line: line,
        col: col,
    };
    let s = format!("Parse Error: {}:{} {}", line, col, pe.msg);
    Err(ErrorKind::Parse(pe, s).into())
}
