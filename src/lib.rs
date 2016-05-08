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

use error::*;

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

trait Formatter {
    /// Called when serializing a '('.
    fn open<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write;

    /// Called when serializing a ' '.
    fn space<W>(&mut self, writer: &mut W, first:bool) -> Result<()>
        where W: io::Write;

    /// Called when serializing a ')'.
    fn close<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write;
}

struct CompactFormatter;

impl Formatter for CompactFormatter {
    fn open<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b"(").map_err(From::from)
    }
    fn space<W>(&mut self, writer: &mut W, first:bool) -> Result<()>
        where W: io::Write
    {
        if first {
            Ok(())
        } else {
            writer.write_all(b" ").map_err(From::from)
        }
    }
    fn close<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b")").map_err(From::from)
    }
}


struct Serializer<W, F=CompactFormatter> {
    writer: W,
    formatter: F,

    /// `first` is used to signify if we should print a comma when we are walking through a
    /// sequence.
    first: bool,
}

// dispatches only based on Formatter
impl<W> Serializer<W>
    where W: io::Write,
{
    fn new(writer: W) -> Self {
        Serializer::with_formatter(writer, CompactFormatter)
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
            first: false,
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
                try!(self.formatter.open(&mut self.writer));
                let mut first = true;
                for v in list {
                    try!(self.formatter.space(&mut self.writer, first));
                    first = false;
                    try!(self.serialize(v));
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


pub fn to_vec(value:&Sexp) -> Result<Vec<u8>> {
    let mut writer = Vec::with_capacity(128);
    try!(to_writer(&mut writer, value));
    Ok(writer)
}

pub fn to_string(value:&Sexp) -> Result<String> {
    let vec = try!(to_vec(value));
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


// internal tests
#[test]
fn test_qstring1() {
    let x = parse_string(&b"\"hello world\""[..]);
    match x {
        nom::IResult::Done(_,y) => {
            match y {
                Sexp::String(f) => assert_eq!(String::from("hello world"), f),
                _ => panic!("not string"),
            }
        },
        _ => panic!("parser not done"),
    }
}

/*
#[test]
#[should_panic(expected="assertion failed: `(left == right)` (left: `Incomplete(Size(1))`, right: `Done([], \"hello\")`)")]
fn test_qstring2() {
    parse_string(&b"\"hello"[..]);
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn check_parse_res(s: &str, o:&str) {
        let e = parse_str(s).unwrap();
        let t = to_string(&e).unwrap();
        assert_eq!(o, t)
    }
    
    #[allow(dead_code)]
    fn check_parse(s: &str) {
        let e = parse_str(s).unwrap();
        let t = to_string(&e).unwrap();
        assert_eq!(s, t)
    }

    #[allow(dead_code)]
    fn parse_fail(s: &str) {
        parse_str(s).unwrap();
    }

    #[test]
    fn test_empty() { check_parse("") }
    
    #[test]
    fn test_empty_qstring() { check_parse("(hello \"\")") }

    #[test]
    fn test_minimal() { check_parse("()") }

    #[test]
    fn test_string() { check_parse("hello") }

    #[test]
    fn test_qstring_a() { check_parse_res("\"hello\"", "hello") }
    
    #[test]
    fn test_qstring_a2() { check_parse("\"hello world\"") }
    
    #[test]
    fn test_qstring_a3() { check_parse("\"hello(world)\"") }

    #[test]
    fn test_number() { check_parse("1.3") }
    
    #[test]
    fn test_float_vs_int() { check_parse("2.0") }

    #[test]
    fn test_double() { check_parse("(())") }

    #[test]
    fn test_br_string() { check_parse("(world)") }

    #[test]
    fn test_br_qstring() { check_parse_res("(\"world\")", "(world)") }

    #[test]
    fn test_br_int() { check_parse("(42)") }

    #[test]
    fn test_br_float() { check_parse("(12.7)") }
    
    #[test]
    fn test_br_qbrstring() { check_parse("(\"(()\")") }
    
    #[test]
    fn test_number_string() { check_parse("567A_WZ") }
    
    #[test]
    #[should_panic(expected="called `Result::unwrap()` on an `Err` value: Other(\"incomplete: Size(2)\")")]
    fn test_invalid1() { parse_fail("(") }

    #[test]
    #[should_panic(expected="called `Result::unwrap()` on an `Err` value: Other(\"parse error: Alt |)|\")")]
    fn test_invalid2() { parse_fail(")") }

    #[test]
    #[should_panic(expected="incomplete: Size")]
    fn test_invalid3() { parse_fail("\"hello") }

    #[test]
    fn test_complex() { check_parse("(module SWITCH_3W_SIDE_MMP221-R (layer F.Cu) (descr \"\") (pad 1 thru_hole rect (size 1.2 1.2) (at -2.5 -1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (pad 2 thru_hole rect (size 1.2 1.2) (at 0.0 -1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (pad 3 thru_hole rect (size 1.2 1.2) (at 2.5 -1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (pad 5 thru_hole rect (size 1.2 1.2) (at 0.0 1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (pad 6 thru_hole rect (size 1.2 1.2) (at -2.5 1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (pad 4 thru_hole rect (size 1.2 1.2) (at 2.5 1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (fp_line (start -4.5 -1.75) (end 4.5 -1.75) (layer F.SilkS) (width 0.127)) (fp_line (start 4.5 -1.75) (end 4.5 1.75) (layer F.SilkS) (width 0.127)) (fp_line (start 4.5 1.75) (end -4.5 1.75) (layer F.SilkS) (width 0.127)) (fp_line (start -4.5 1.75) (end -4.5 -1.75) (layer F.SilkS) (width 0.127)))") }

    #[test]
    fn test_multiline() {
        check_parse("\
(hello \"test it\"
    (foo bar)
    (mars venus)
)
")
    }

    #[test]
    fn test_multiline2() {
        check_parse("\
(hello world
  mad
    (world)
  not)")
    }

    #[test]
    fn test_multiline_two_empty() {
        check_parse("\
(hello

world)")
    }

    #[test]
    fn test_fail_pcb() {
        check_parse("\
(kicad_pcb (version 4) (host pcbnew \"(2015-05-31 BZR 5692)-product\")
  (general
  )
)")
    }
}

pub mod error;
