// (c) 2016 Productize SPRL <joost@productize.be>

use Result;
use Sexp;
use parse_error;
use str_error;
use std::result;
use std::io;
use std::fs::File;
use std::io::prelude::*;

#[derive(Default)]
struct Parser {
    data: Vec<char>,
    position: usize,
    line: usize,
    line_position: usize,
}

impl Parser {
    fn peek(&self) -> Result<char> {
        try!(self.fail_on_eof());
        Ok(self.data[self.position])
    }

    fn get(&mut self) -> Result<char> {
        try!(self.fail_on_eof());
        let c = self.data[self.position];
        self.position += 1;
        self.line_position += 1;
        if c == '\n' {
            self.line += 1;
            self.line_position = 0;
        }
        Ok(c)
    }

    fn inc(&mut self) {
        let c = self.data[self.position];
        self.position += 1;
        self.line_position += 1;
        if c == '\n' {
            self.line += 1;
            self.line_position = 0;
        }
    }

    fn eat_space(&mut self) {
        while !self.eof() {
            let c = self.data[self.position];
            if c == ' ' || c == '\t' {
                self.inc();
                continue;
            }
            break;
        }
    }

    fn eat_char(&mut self, c: char) -> Result<()> {
        let c2 = try!(self.get());
        if c != c2 {
            self.parse_error(&format!("expected {} got {}", c, c2))
        } else {
            Ok(())
        }
    }

    fn eof(&self) -> bool {
        self.position >= self.data.len()
    }

    fn fail_on_eof(&self) -> Result<()> {
        if self.eof() {
            return self.parse_error("End of file reached");
        }
        Ok(())
    }

    fn parse_error<T>(&self, msg: &str) -> Result<T> {
        parse_error(self.line + 1, self.line_position + 1, msg.to_string())
    }
}

/// parse a &str to a symbolic-expression
pub fn parse_str(sexp: &str) -> Result<Sexp> {
    if sexp.is_empty() {
        return Ok(Sexp::new_empty());
    }
    let mut parser = Parser::default();
    parser.data = sexp.chars().collect();
    parse(&mut parser)
}

fn parse(parser: &mut Parser) -> Result<Sexp> {
    parser.eat_space();
    let c = try!(parser.peek());
    if c == '(' {
        parse_list(parser)
    } else if c == '"' {
        parse_quoted_string(parser)
    } else if c == ')' {
        parser.parse_error("Unexpected )")
    } else {
        parse_bare_string(parser)
    }
}

fn parse_list(parser: &mut Parser) -> Result<Sexp> {
    try!(parser.eat_char('('));
    let mut v = vec![];
    while !parser.eof() {
        let c = try!(parser.peek());
        if c == ')' {
            break;
        } else if c == ' ' || c == '\t' || c == '\r' || c == '\n' {
            parser.inc()
        } else {
            let s = try!(parse(parser));
            v.push(s)
        }
    }
    try!(parser.eat_char(')'));
    parser.eat_space();
    Ok(Sexp::List(v))
}

fn parse_quoted_string(parser: &mut Parser) -> Result<Sexp> {
    let mut s = String::new();
    try!(parser.eat_char('"'));
    // note that escaped quotes are actually not allowed
    while !parser.eof() {
        let c = try!(parser.peek());
        if c == '"' {
            break;
        }
        s.push(c);
        parser.inc()
    }
    try!(parser.eat_char('"'));
    Ok(Sexp::String(s))
}

fn parse_bare_string(parser: &mut Parser) -> Result<Sexp> {
    let mut s = String::new();
    while !parser.eof() {
        let c = try!(parser.peek());
        if c == ' ' || c == '(' || c == ')' || c == '\r' || c == '\n' {
            break;
        }
        s.push(c);
        parser.inc()
    }
    Ok(Sexp::String(s))
}

fn read_file(name: &str) -> result::Result<String, io::Error> {
    let mut f = try!(File::open(name));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    Ok(s)
}

/// parse a file as a symbolic-expression
pub fn parse_file(name: &str) -> Result<Sexp> {
    let s = try!(match read_file(name) {
        Ok(s) => Ok(s),
        Err(x) => str_error(format!("{:?}", x)),
    });
    parse_str(&s[..])
}
