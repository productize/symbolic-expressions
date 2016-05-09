#![feature(plugin)]
#![plugin(clippy)]

#[macro_use]
extern crate nom;

use std::str;
use std::str::FromStr;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io;

pub use error::*;
use formatter::*;

// like Into trait but works from a ref avoiding consumption or expensive clone
pub trait IntoSexp {
    fn into_sexp(&self) -> Sexp;
}

#[derive(Debug, Clone)]
pub enum Sexp {
    String(String),
    List(Vec<Sexp>),
    Empty,
}

impl Sexp {

    pub fn new_empty() -> Sexp {
        Sexp::Empty
    }

    pub fn from<T:IntoSexp>(t:&T) -> Sexp {
        t.into_sexp()
    }
    
    pub fn list(&self) -> Result<&Vec<Sexp> > {
        match *self {
            Sexp::List(ref v) => Ok(v),
            _ => str_error(format!("not a list: {}", self))
        }
    }
    
    pub fn string(&self) -> Result<&String> {
        match *self {
            Sexp::String(ref s) => Ok(s),
            _ => str_error(format!("not a string: {}", self))
        }
    }

    pub fn f(&self) -> Result<f64> {
        let s = try!(self.string());
        match f64::from_str(&s) {
            Ok(f) => Ok(f),
            _ => str_error("Error parsing float".to_string())
        }
    }

    pub fn i(&self) -> Result<i64> {
        let s = try!(self.string());
        match i64::from_str(&s) {
            Ok(f) => Ok(f),
            _ => str_error("Error parsing int".to_string())
        }
    }
    
    pub fn list_name(&self) -> Result<&String> {
        let l = try!(self.list());
        let l = &l[..];
        let a = try!(l[0].string());
        Ok(a)
    }

    pub fn slice_atom(&self, s:&str) -> Result<&[Sexp]> {
        let v = try!(self.list());
        let v2 =&v[..];
        let st = try!(v2[0].string());
        if st != s {
            return str_error(format!("list {} doesn't start with {}, but with {}", self, s, st))
        };
        Ok(&v[1..])
    }

    pub fn named_value(&self, s:&str) -> Result<&Sexp> {
        let v = try!(self.list());
        if v.len() != 2 {
            return str_error(format!("list {} is not a named_value", s))
        }
        let l = try!(self.slice_atom(s));
        Ok(&l[0])
    }

    pub fn named_value_i(&self, s:&str) -> Result<i64> {
        try!(self.named_value(s)).i()
    }
    
    pub fn named_value_f(&self, s:&str) -> Result<f64> {
        try!(self.named_value(s)).f()
    }
    
    pub fn named_value_string(&self, s:&str) -> Result<&String> {
        try!(self.named_value(s)).string()
    }
    
    pub fn slice_atom_num(&self, s:&str, num:usize) -> Result<&[Sexp]> {
        let v = try!(self.list());
        let v2 =&v[..];
        let st = try!(v2[0].string());
        if st != s {
            return str_error(format!("list doesn't start with {}, but with {}", s, st))
        };
        if v.len() != (num+1) {
            return str_error(format!("list ({}) doesn't have {} elements but {}", s, num, v.len()-1))
        }
        Ok(&v[1..])      
    }
}

impl fmt::Display for Sexp {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        match *self {
            Sexp::String(ref s) => {
                if s.contains('(') || s.contains(' ') {
                    write!(f,"\"{}\"", s)
                } else {
                    write!(f,"{}", s)
                }
            },
            Sexp::List(ref v) => {
                try!(write!(f, "("));
                let l = v.len();
                for (i,x) in v.iter().enumerate() {
                    if i < l-1 {
                        try!(write!(f, "{} ", x));
                    } else {
                        try!(write!(f, "{}", x));
                    }
                }
                write!(f, ")")
            },
            Sexp::Empty => Ok(())
        }
    }
}

struct Serializer<W, F=CompactFormatter> {
    writer: W,
    formatter: F,
}

// dispatches only based on Formatter
impl<W> Serializer<W>
    where W: io::Write,
{
    fn new(writer: W) -> Self {
        Serializer::with_formatter(writer, CompactFormatter)
    }
}

impl<W> Serializer<W, RulesFormatter>
    where W: io::Write,
{
    /* TODO
    fn new_rules(writer: W) -> Self {
        Serializer::with_formatter(writer, RulesFormatter::new())
    }
     */
    
    fn new_kicad(writer: W) -> Self {
        Serializer::with_formatter(writer, RulesFormatter::new_kicad())
    }
}

impl<W, F> Serializer<W, F>
    where W: io::Write,
          F: Formatter,
{
    fn with_formatter(writer: W, formatter: F) -> Self {
        Serializer {
            writer: writer,
            formatter: formatter,
        }
    }

    fn serialize_str(&mut self, value:&str) -> Result<()> {
        if value.contains('(') || value.contains(' ') || value.is_empty() {
            write!(&mut self.writer, "\"{}\"", value).map_err(From::from)
        } else {
            write!(&mut self.writer, "{}", value).map_err(From::from)
        }
    }

    fn serialize(&mut self, value:&Sexp) -> Result<()> {
        match *value {
            Sexp::String(ref s) => {
                self.serialize_str(s)
            },
            Sexp::List(ref list) => {
                let mut first = true;
                if list.is_empty() {
                    try!(self.formatter.open(&mut self.writer, None));
                } else {                   
                    for v in list {
                        if first {
                            try!(self.formatter.open(&mut self.writer, Some(v)));
                            try!(self.serialize(v));
                            first = false;
                        } else {
                            try!(self.formatter.element(&mut self.writer, v));
                            try!(self.serialize(v));
                        }
                    }
                }
                self.formatter.close(&mut self.writer)
            },
            Sexp::Empty => Ok(()),
        }
        
    }
}


pub fn parse_str(sexp: &str) -> Result<Sexp> {
    if sexp.is_empty() {
        return Ok(Sexp::new_empty())
    }
    match parse_sexp(&sexp.as_bytes()[..]) {
        nom::IResult::Done(_, c) => Ok(c),
        nom::IResult::Error(err) => {
            match err {
                nom::Err::Position(kind,p) => 
                    str_error(format!("parse error: {:?} |{}|", kind, str::from_utf8(p).unwrap())),
                _ => str_error("parse error".to_string())
            }
        },
        nom::IResult::Incomplete(x) => str_error(format!("incomplete: {:?}", x)),
    }
}

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
    parse_str(&s[..])
}

pub fn to_writer<W>(writer: &mut W, value: &Sexp) -> Result<()>
    where W: io::Write
{
    let mut ser = Serializer::new(writer);
    ser.serialize(value)
}

pub fn to_kicad_writer<W>(writer: &mut W, value: &Sexp) -> Result<()>
    where W: io::Write
{
    let mut ser = Serializer::new_kicad(writer);
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


named!(parse_qstring<String>,
       map_res!(
           map_res!(
               delimited!(char!('\"'), is_not!("\""), char!('\"')),
               str::from_utf8),
           FromStr::from_str)
       );

named!(parse_bare_string<String>,
       map_res!(
           map_res!(
               is_not!(b")( \r\n"),
               str::from_utf8),
           FromStr::from_str)
       );

named!(parse_string<Sexp>,
       map!(alt!(parse_qstring | parse_bare_string), |x| Sexp::String(x))
       );

named!(parse_list<Sexp>,
       chain!(
           char!('(') ~
               v: many0!(parse_sexp) ~
               _space: opt!(nom::multispace) ~ // sometimes there is space after a closing bracket, this would not be caught by parse_sexp
               char!(')'),
           || Sexp::List(v) )
       );

// TODO: consider lines with just spaces and a nl as also nl
named!(line_ending<usize>,
       chain!(
           opt!(nom::space) ~
               c: opt!(is_a!(b"\r\n"))
               , || match c { None => 0, Some(ref x) => x.len(), }
               )
       );

named!(parse_sexp<Sexp>,
           chain!(
               _indent: opt!(nom::space) ~
                   sexp: alt!(parse_list | parse_string) ~
                   _nl: line_ending
                   ,
               || {
                   sexp
               })
       );

mod error;
mod formatter;

#[cfg(test)]
mod test;
