// (c) 2015 Joost Yervante Damad <joost@damad.be>

// loosely based on https://github.com/cgaebel/sexp
// latest version can be found at https://github.com/andete/rust_sexp

use std::fmt;
use std::str::FromStr;

use std::fs::File;
use std::io::prelude::*;

pub enum Atom {
  S(String),
  Q(String),
  I(i64),
  F(f64),
}

pub enum Sexp {
  Atom(Atom),
  Empty,
  List(Vec<Sexp>),
}

pub struct Error {
    msg: &'static str,
    line: usize,
    lpos: usize,
}

type Err = Box<Error>;
type ERes<T> = Result<T, Err>;

fn err<T>(msg: &'static str, state: &ParseState) -> ERes<T> {
    Err(Box::new(Error { msg: msg, line: state.line, lpos: state.lpos }))
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
            return err("end of document reached", self)
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
            return err("end of document reached", self)
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

    fn take(&mut self) -> ERes<char> {
        let c = try!(self.peek());
        try!(self.next());
        Ok(c)
    }
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Parse Error {}:{}: {}", self.line, self.lpos, self.msg)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Parse Error {}:{}: {}", self.line, self.lpos, self.msg)
    }
}

impl fmt::Display for Atom {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Atom::S(ref s) => write!(f, "{}", s),
      Atom::Q(ref s) => write!(f, "\"{}\"", s),
      Atom::I(i)     => write!(f, "{}", i),
      Atom::F(d)     => write!(f, "{}", d),
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
                    '"' => return err("unexpected \" in string", state),
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
                        return err("unexpected char in number", state)
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
                return err("unmatched )", state)
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

fn read_file(name: &str) -> Result<String,std::io::Error> {
    let mut f = try!(File::open(name));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    Ok(s)
}

pub fn parse_str(s: &str) -> Sexp {
    parse(s).unwrap()
}

pub fn parse_file(name: &str) -> Sexp {
    let s = read_file(name).unwrap();
    parse(&s[..]).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn check_parse(s: &str) {
        let e = parse_str(s);
        let t = format!("{}", e);
        assert_eq!(s, t);
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
    #[should_panic(expected="Parse Error 1:1: end of document reached")]
    fn test_invalid1() { parse_str("("); }

    #[test]
    #[should_panic(expected="Parse Error 1:0: unmatched )")]
    fn test_invalid2() { parse_str(")"); }

    #[test]
    #[should_panic(expected="Parse Error 1:6: end of document reached")]
    fn test_invalid3() { parse_str("\"hello"); }
}
