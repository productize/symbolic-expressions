// (c) 2015 Joost Yervante Damad <joost@damad.be>

// loosely based on https://github.com/cgaebel/sexp
// latest version can be found at https://github.com/andete/rust_sexp

use std::fmt;
use std::str::FromStr;
use std::f64;

use std::fs::File;
use std::io::prelude::*;

pub enum Atom {
  S(String),
  Q(String),
  I(i64),
  F(f64),
}

impl Atom {
    pub fn f(&self) -> Result<f64,String> {
        match *self {
            Atom::F(f) => Ok(f),
            ref x => Err(format!("not a float: {}", x))
        }
    }
    pub fn i(&self) -> Result<i64,String> {
        match *self {
            Atom::I(i) => Ok(i),
            ref x => Err(format!("not an int: {}", x))
        }
    }
    pub fn string(&self) -> Result<String,String> {
        match *self {
            Atom::S(ref s) => Ok(s.clone()),
            Atom::Q(ref s) => Ok(s.clone()),
            ref x => Err(format!("not a string: {}", x))
        }
    }
    pub fn as_string(&self) -> Result<String,String> {
        match *self {
            Atom::S(ref s) => Ok(s.clone()),
            Atom::Q(ref s) => Ok(s.clone()),
            Atom::F(ref s) => Ok(format!("{}", s)),
            Atom::I(ref s) => Ok(format!("{}", s)),
        }
    }
}


pub enum Sexp {
  Atom(Atom),
  Empty,
  List(Vec<Sexp>),
}

impl Sexp {
    pub fn atom(&self) -> Result<&Atom,String> {
        match *self {
            Sexp::Atom(ref f) => Ok(f),
            ref x => Err(format!("not an atom: {}", x))
        }
    }
    pub fn list(&self) -> Result<&Vec<Sexp>,String> {
        match *self {
            Sexp::List(ref v) => Ok(v),
            ref x => Err(format!("not a list: {}", x))
        }
    }
}

pub struct ParseError {
    msg: &'static str,
    line: usize,
    lpos: usize,
}

pub enum Error {
    ParseError(ParseError),
    IOError(std::io::Error),
}

pub type ERes<T> = Result<T, Error>;

fn err_parse<T>(msg: &'static str, state: &ParseState) -> ERes<T> {
    Err(Error::ParseError(ParseError { msg: msg, line: state.line, lpos: state.lpos}))
}

fn err_io<T>(err: std::io::Error) -> ERes<T> {
    Err(Error::IOError(err))
}

struct ParseState {
    pos: usize,
    line: usize,
    lpos: usize,
    vec: Vec<char>,
    len: usize,
}

impl ParseState {
    fn peek(&self) -> ERes<char> {
        if self.pos >= self.len {
            return err_parse("end of document reached", self)
        }
        Ok(self.vec[self.pos])
    }
    fn peek_option(&self) -> Option<char> {
        if self.pos >= self.len {
            return None
        }
        Some(self.vec[self.pos])
    }
    fn next(&mut self) -> ERes<()> {
        if self.pos >= self.len {
            return err_parse("end of document reached", self)
        }
        let c = self.vec[self.pos];
        self.pos += 1;
        match c {
            '\r' | '\n' => {
                self.lpos = 0;
                self.line += 1;
            }
            _ => {
                self.lpos += 1;
            }
        }
        Ok(())
    }
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::ParseError(ref p) =>
                write!(f, "Parse Error {}:{}: {}", p.line, p.lpos, p.msg),
            Error::IOError(ref i) =>
                write!(f, "{}", i)
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::ParseError(ref p) =>
                write!(f, "Parse Error {}:{}: {}", p.line, p.lpos, p.msg),
            Error::IOError(ref i) =>
                write!(f, "{}", i)
        }
    }
}

impl fmt::Display for Atom {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
        Atom::S(ref s) => write!(f, "{}", s),
        Atom::Q(ref s) => write!(f, "\"{}\"", s),
        Atom::I(i)     => write!(f, "{}", i),
        Atom::F(d)     => {
            let z = d.floor();
            if d - z < f64::EPSILON {
                write!(f, "{}.0", z)
            } else {
                write!(f, "{}", d)
            }
        }    
    }
  }
}

impl fmt::Display for Sexp {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Sexp::Atom(ref a) => write!(f, "{}", a),
      Sexp::List(ref xs) => {
        try!(write!(f, "("));
        for (i, x) in xs.iter().enumerate() {
          let s = if i == 0 { "" } else { " " };
          try!(write!(f, "{}{}", s, x));
        }
        write!(f, ")")
      },
      Sexp::Empty => write!(f, ""),
    }
  }
}

fn parse_list(state: &mut ParseState) -> ERes<Sexp> {
    //println!("list");
    try!(state.next()); // skip (
    let mut l: Vec<Sexp> = Vec::new();
    loop {
        match try!(state.peek()) {
            ')' => {
                try!(state.next());
                break;
            }
            _ => {
                l.push(try!(parse_sexp(state)));
            }
        }
    }
    //println!("Found list");
    Ok(Sexp::List(l))
}

fn parse_quoted_string(state: &mut ParseState) -> ERes<Atom> {
    //println!("qstring");
    try!(state.next()); // skip "
    let mut s = String::new();
    loop {
        match try!(state.peek()) {
            '"' => {
                try!(state.next());
                break
            }
            x @ '\r' | x @ '\n' => {
                s.push(x);
                try!(state.next());
                }
            x => {
                s.push(x);
                try!(state.next());
            } 
        }
    }
    //println!("Found quoted string {}", s);
    Ok(Atom::Q(s))
}

fn parse_string(state: &mut ParseState) -> ERes<Atom> {
    //println!("string");
    let mut s = String::new();
    loop {
        match state.peek_option() {
            Some(x) => {
                match x {
                    ' ' | '\t' | '\r' | '\n' | ')' => break,
                    '"' => return err_parse("unexpected \" in string", state),
                    x => s.push(x),
                }
            }
            None => {
                break;
            }
        }
        try!(state.next())
    }
    //println!("Found string {}", s);
    Ok(Atom::S(s))
}

fn parse_number(state: &mut ParseState) -> ERes<Atom> {
    //println!("number");
    let mut s = String::new();
    loop {
        match state.peek_option() {
            Some(x) => {
                match x {
                    ' ' | '\r' | '\n' | '\t' | ')' => {
                        break
                    },
                    '0' ... '9' | '.' | '-' => {
                        s.push(state.vec[state.pos])
                    },
                    _ => {
                        return err_parse("unexpected char in number", state)
                    },
                }
            }
            None => {
                break
            } 
        }
        try!(state.next())
    }
    //println!("Found number {}", s);
    let s2: &str = &s[..];
    if s.contains('.') {
        Ok(Atom::F(f64::from_str(s2).unwrap()))
    } else {
        Ok(Atom::I(i64::from_str(s2).unwrap()))
    }
}

fn parse_atom(state: &mut ParseState) -> ERes<Sexp> {
    //println!("atom");
    let a = match try!(state.peek()) {
        '"' => {
            try!(parse_quoted_string(state))
        }
        '0' ... '9' | '.' | '-' => {
            try!(parse_number(state))
        }
        _ => {
            try!(parse_string(state))
        }
    };
    Ok(Sexp::Atom(a))
}


fn parse_sexp(state: &mut ParseState) -> ERes<Sexp> {
    loop {
        match try!(state.peek()) {
            '(' => {
                return parse_list(state)
            }
            ' ' | '\t' | '\r' | '\n' => {
                try!(state.next());
            }
            ')' => {
                return err_parse("unmatched )", state)
            }
            _ => {
                return parse_atom(state)
            }
        }
    }
}

fn parse(data: &str) -> ERes<Sexp> {
    if data.len() == 0 {
        Ok(Sexp::Empty)
    } else {
        let vec: Vec<char> = data.chars().collect();
        let len = vec.len();
        let state = &mut ParseState { pos: 0, line: 1, lpos: 0, vec: vec, len: len };
        parse_sexp(state)
    }
}

fn read_file(name: &str) -> Result<String, std::io::Error> {
    let mut f = try!(File::open(name));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    Ok(s)
}

pub fn parse_str(s: &str) -> ERes<Sexp> {
    parse(s)
}

pub fn parse_file(name: &str) -> ERes<Sexp> {
    let s = try!(match read_file(name) {
        Ok(s) => Ok(s),
        Err(x) => err_io(x),
    });
    parse(&s[..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn check_parse(s: &str) {
        let e = parse_str(s).unwrap();
        let t = format!("{}", e);
        assert_eq!(s, t);
    }

    #[allow(dead_code)]
    fn parse_fail(s: &str) {
        parse_str(s).unwrap();
    }
    

    #[test]
    fn test_empty() { check_parse("") }

    #[test]
    fn test_minimal() { check_parse("()") }

    #[test]
    fn test_string() { check_parse("hello") }

    #[test]
    fn test_qstring() { check_parse("\"hello\"") }

    #[test]
    fn test_number() { check_parse("1.3") }
    
    #[test]
    fn test_float_vs_int() { check_parse("2.0") }

    #[test]
    fn test_double() { check_parse("(())") }

    #[test]
    fn test_br_string() { check_parse("(world)") }

    #[test]
    fn test_br_qstring() { check_parse("(\"world\")") }

    #[test]
    fn test_br_int() { check_parse("(42)") }

    #[test]
    fn test_br_float() { check_parse("(12.7)") }
    
    #[test]
    fn test_br_qbrstring() { check_parse("(\"(()\")") }
    
    #[test]
    #[should_panic(expected="Parse Error 1:1: end of document reached")]
    fn test_invalid1() { parse_fail("(") }

    #[test]
    #[should_panic(expected="Parse Error 1:0: unmatched )")]
    fn test_invalid2() { parse_fail(")") }

    #[test]
    #[should_panic(expected="Parse Error 1:6: end of document reached")]
    fn test_invalid3() { parse_fail("\"hello") }

    #[test]
    fn test_complex() { check_parse("(module SWITCH_3W_SIDE_MMP221-R (layer F.Cu) (descr \"\") (pad 1 thru_hole rect (size 1.2 1.2) (at -2.5 -1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (pad 2 thru_hole rect (size 1.2 1.2) (at 0.0 -1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (pad 3 thru_hole rect (size 1.2 1.2) (at 2.5 -1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (pad 5 thru_hole rect (size 1.2 1.2) (at 0.0 1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (pad 6 thru_hole rect (size 1.2 1.2) (at -2.5 1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (pad 4 thru_hole rect (size 1.2 1.2) (at 2.5 1.6 0) (layers *.Cu *.Mask) (drill 0.8)) (fp_line (start -4.5 -1.75) (end 4.5 -1.75) (layer F.SilkS) (width 0.127)) (fp_line (start 4.5 -1.75) (end 4.5 1.75) (layer F.SilkS) (width 0.127)) (fp_line (start 4.5 1.75) (end -4.5 1.75) (layer F.SilkS) (width 0.127)) (fp_line (start -4.5 1.75) (end -4.5 -1.75) (layer F.SilkS) (width 0.127)))") }
}
