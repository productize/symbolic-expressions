// (c) 2016 Productize SPRL <joost@productize.be>

use Result;
use Sexp;
use parse_error;

#[derive(Default)]
struct Parser {
    data: Vec<char>,
    position: usize,
    line: usize,
    line_position: usize,
}

impl Parser {
    fn peek(&self) -> Result<char> {
        try!(self.eof());
        Ok(self.data[self.position])
    }

    fn get(&mut self) -> Result<char> {
        try!(self.eof());
        let c = self.data[self.position];
        self.position += 1;
        if c == '\n' {
            self.line += 1;
            self.line_position = 0;
        }
        Ok(c)
    }

    fn eat(&mut self) -> Result<()> {
        let _:char = try!(self.get());
        Ok(())
    }
    
    fn eof(&self) -> Result<()> {
        if self.position >= self.data.len() {
            try!(self.parse_error("end of file reached").into())
        }
        Ok(())
    }
    
    fn parse_error<T>(&self, msg:&str) -> Result<T> {
        parse_error(self.line, self.line_position, msg.to_string())
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

fn parse(parser:&mut Parser) -> Result<Sexp> {
    try!(eat_space(parser));
    // TODO
    Ok(Sexp::new_empty())
}

fn eat_space(parser:&mut Parser) -> Result<()> {
    loop {
        let c = try!(parser.peek());
        if c == ' ' || c == '\t' {
            try!(parser.eat());
            continue
        }
        break
    }
    Ok(())
}
