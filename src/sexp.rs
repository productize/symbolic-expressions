// (c) 2016 Productize SPRL <joost@productize.be>

use std::fmt;
use std::result;
use std::str::FromStr;

use str_error;
use Result;

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

/*

Following the KiCad file formats specification chapter 4.4 - Identifiers and Strings:
A string may not contain an actual newline or carriage return, but it may use an escape sequence to encode a
newline, such as \n.
If a string has any of the following conditions, then it must be quoted with a leading and trailing double quote
character, otherwise it is acceptable to not quote the string:
1. has one or more of the following 4 separator bytes: ASCII space, tab, '(', or ')'.
2. has one or more of the following bytes: '%', '{', or '}'.
3. has a length of zero bytes, and you need a place holder for the field, then use "".
4. includes a byte of '-', and this byte is not in the first position of the string.
*/

pub fn encode_string(s:&str) -> String {
    fn rule_4(s:&str) -> bool {
        s.contains('-') && s.len() > 1 && s.as_bytes()[0] != 45
    }
    if s.contains('(') || s.contains(' ') || s.contains(')') || s.contains('\t') || s.contains('{') || s.contains('}') || s.contains('}') || s.contains('%') || s.is_empty() || rule_4(s) {
        format!("\"{}\"", s)
    } else {
        String::from(s)
    }
}

impl Sexp {

    pub fn new_empty() -> Sexp {
        Sexp::Empty
    }

    pub fn new_string(s:&str) -> Sexp {
        Sexp::String(s.to_string())
    }

    pub fn new_list(v:Vec<Sexp>) -> Sexp {
        Sexp::List(v)
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
        match f64::from_str(s) {
            Ok(f) => Ok(f),
            _ => str_error("Error parsing float".to_string())
        }
    }

    pub fn i(&self) -> Result<i64> {
        let s = try!(self.string());
        match i64::from_str(s) {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match *self {
            Sexp::String(ref s) => {
                write!(f, "{}", encode_string(s))
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
