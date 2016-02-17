#[macro_use]
extern crate nom;

use std::str;
use std::str::FromStr;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

// TODO: store formatting hints in Sexp
#[derive(Debug, Clone)]
pub enum Sexp {
    String(String),
    List(Vec<Sexp>),
    Empty,
}

pub type ERes<T> = Result<T, String>;

impl Sexp {
    pub fn list(&self) -> ERes<&Vec<Sexp> > {
        match *self {
            Sexp::List(ref v) => Ok(v),
            ref x => Err(format!("not a list: {}", x))
        }
    }
    
    pub fn string(&self) -> ERes<&String> {
        match *self {
            Sexp::String(ref s) => Ok(s),
            ref x => Err(format!("not a string: {}", x))
        }
    }

    pub fn f(&self) -> ERes<f64> {
        let s = try!(self.string());
        match f64::from_str(&s) {
            Ok(f) => Ok(f),
            _ => Err(format!("Error parsing float"))
        }
    }

    pub fn i(&self) -> ERes<i64> {
        let s = try!(self.string());
        match i64::from_str(&s) {
            Ok(f) => Ok(f),
            _ => Err(format!("Error parsing int"))
        }
    }
    
    pub fn list_name(&self) -> ERes<&String> {
        let l = try!(self.list());
        let l = &l[..];
        let a = try!(l[0].string());
        Ok(a)
    }

    pub fn slice_atom(&self, s:&str) -> ERes<&[Sexp]> {
        let v = try!(self.list());
        let v2 =&v[..];
        let st = try!(v2[0].string());
        if st != s {
            return Err(format!("list doesn't start with {}, but with {}", s, st))
        };
        Ok(&v[1..])
    }
}

impl fmt::Display for Sexp {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Sexp::String(ref s) => {
                if s.contains("(") || s.contains(" ") {
                    write!(f,"\"{}\"", s)
                } else {
                    write!(f,"{}", s)
                }
            },
            Sexp::List(ref v) => {
                try!(write!(f, "("));
                for (i, x) in v.iter().enumerate() {
                    let s = if i == 0 { "" } else { " " };
                    try!(write!(f, "{}{}", s, x));
                }
                write!(f, ")")
            },
            Sexp::Empty => Ok(())
        }
    }
}

pub fn display_string(s:&String) -> String {
    if s.contains("(") || s.contains(" ") || s.len() == 0 {
        format!("\"{}\"", s)
    } else {
        s.clone()
    }
}

pub fn parse_str(sexp: &str) -> Result<Sexp, String> {
    if sexp.len() == 0 {
        return Ok(Sexp::Empty)
    }
    match parse_sexp(&sexp.as_bytes()[..]) {
        nom::IResult::Done(_, c) => Ok(c),
        nom::IResult::Error(err) => {
            match err {
                nom::Err::Position(kind,p) => 
                    Err(format!("parse error: {:?} |{}|", kind, str::from_utf8(p).unwrap())),
                _ => Err(format!("parse error"))
            }
        },
        nom::IResult::Incomplete(x) => Err(format!("incomplete: {:?}", x)),
    }
}

fn read_file(name: &str) -> Result<String, std::io::Error> {
    let mut f = try!(File::open(name));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    Ok(s)
}

pub fn parse_file(name: &str) -> ERes<Sexp> {
    let s = try!(match read_file(name) {
        Ok(s) => Ok(s),
        Err(x) => Err(format!("{:?}", x))
    }); 
    parse_str(&s[..])
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

named!(parse_string<String>,
       alt!(parse_qstring | parse_bare_string)
       );

named!(parse_list<Vec<Sexp> >,
       chain!(
           char!('(') ~
           v: many0!(parse_sexp) ~
           char!(')'),
           || v)
       );

named!(parse_sexp<Sexp>,
       chain!(
           opt!(nom::multispace) ~
               s: alt!(map!(parse_list,Sexp::List) | map!(parse_string,Sexp::String)) ~
               opt!(nom::multispace)
               ,|| s)
       );


// internal tests
#[test]
fn test_qstring1() {
    assert_eq!(parse_string(&b"\"hello world\""[..]), nom::IResult::Done(&b""[..], String::from("hello world")));
}

#[test]
#[should_panic(expected="assertion failed: `(left == right)` (left: `Incomplete(Size(1))`, right: `Done([], \"hello\")`)")]
fn test_qstring2() {
    assert_eq!(parse_string(&b"\"hello"[..]), nom::IResult::Done(&b""[..], String::from("hello")));
}

#[test]
fn test_string1() {
    assert_eq!(parse_string(&b"hello world"[..]), nom::IResult::Done(&b" world"[..], String::from("hello")));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn check_parse_res(s: &str, o:&str) {
        let e = parse_str(s).unwrap();
        let t = format!("{}", e);
        assert_eq!(o, t)
    }
    #[allow(dead_code)]
    fn check_parse(s: &str) {
        let e = parse_str(s).unwrap();
        let t = format!("{}", e);
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
    #[should_panic(expected="called `Result::unwrap()` on an `Err` value: \"incomplete: Size(2)\"")]
    fn test_invalid1() { parse_fail("(") }

    #[test]
    #[should_panic(expected="called `Result::unwrap()` on an `Err` value: \"parse error: Alt |)|\"")]
    fn test_invalid2() { parse_fail(")") }

    #[test]
    #[should_panic(expected="called `Result::unwrap()` on an `Err` value: \"incomplete: Size(1)\"")]
    fn test_invalid3() { parse_fail("\"hello") }

    #[test]
    fn test_complex() { check_parse("(module SWITCH_3W_SIDE_MMP221-R (layer F.Cu) (descr \"\") (pad 1 thru_hole rect (size 1.2 1.2) (at -2.5 -1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (pad 2 thru_hole rect (size 1.2 1.2) (at 0.0 -1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (pad 3 thru_hole rect (size 1.2 1.2) (at 2.5 -1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (pad 5 thru_hole rect (size 1.2 1.2) (at 0.0 1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (pad 6 thru_hole rect (size 1.2 1.2) (at -2.5 1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (pad 4 thru_hole rect (size 1.2 1.2) (at 2.5 1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (fp_line (start -4.5 -1.75) (end 4.5 -1.75) (layer F.SilkS) (width 0.127)) (fp_line (start 4.5 -1.75) (end 4.5 1.75) (layer F.SilkS) (width 0.127)) (fp_line (start 4.5 1.75) (end -4.5 1.75) (layer F.SilkS) (width 0.127)) (fp_line (start -4.5 1.75) (end -4.5 -1.75) (layer F.SilkS) (width 0.127)))") }
}

