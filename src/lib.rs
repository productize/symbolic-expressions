#![feature(plugin)]
#![plugin(clippy)]

#[macro_use]
extern crate nom;

use std::str;
use std::fs::File;
use std::io::prelude::*;

pub use error::*;
pub use sexp::*;

fn read_file(name: &str) -> std::result::Result<String, std::io::Error> {
    let mut f = try!(File::open(name));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    Ok(s)
}

pub fn parse_file(name: &str) -> Result<Sexp> {
    let s = try!(match read_file(name) {
        Ok(s) => Ok(s),
        Err(x) => str_error(format!("{:?}", x))
    }); 
    parser::parse_str(&s[..])
}

mod error;
mod formatter;
mod parser;
mod ser;
mod sexp;

#[cfg(test)]
mod test;
