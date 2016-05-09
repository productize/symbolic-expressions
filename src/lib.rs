#![feature(plugin)]
#![plugin(clippy)]

#[macro_use]
extern crate nom;

use std::str;
use std::fs::File;
use std::io::prelude::*;
use std::io;

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

pub fn to_writer<W>(writer: &mut W, value: &Sexp) -> Result<()>
    where W: io::Write
{
    let mut ser = ser::Serializer::new(writer);
    ser.serialize(value)
}

pub fn to_kicad_writer<W>(writer: &mut W, value: &Sexp) -> Result<()>
    where W: io::Write
{
    let mut ser = ser::Serializer::new_kicad(writer);
    ser.serialize(value)
}


pub fn to_vec(value:&Sexp) -> Result<Vec<u8>> {
    let mut writer = Vec::with_capacity(128);
    try!(to_writer(&mut writer, value));
    Ok(writer)
}

pub fn to_kicad_vec(value:&Sexp) -> Result<Vec<u8>> {
    let mut writer = Vec::with_capacity(128);
    try!(to_kicad_writer(&mut writer, value));
    Ok(writer)
}

pub fn to_string(value:&Sexp) -> Result<String> {
    let vec = try!(to_vec(value));
    let string = try!(String::from_utf8(vec));
    Ok(string)
}

pub fn to_kicad_string(value:&Sexp) -> Result<String> {
    let vec = try!(to_kicad_vec(value));
    let string = try!(String::from_utf8(vec));
    Ok(string)
}


mod error;
mod formatter;
mod parser;
mod ser;
mod sexp;

#[cfg(test)]
mod test;
