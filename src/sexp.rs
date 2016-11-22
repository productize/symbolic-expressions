// (c) 2016 Productize SPRL <joost@productize.be>

use std::fmt;
use std::result;
use std::str::FromStr;

use str_error;
use Result;

/// like Into trait but works from a ref avoiding consumption or expensive clone
pub trait IntoSexp {
    /// convert self into a Sexp
    fn into_sexp(&self) -> Sexp;
}

/// a symbolic-expression
/// Can be a string or a list or nothing
#[derive(Debug, Clone, PartialEq)]
pub enum Sexp {
    /// plain String symbolic-expression
    String(String),
    /// list symbolic-expression
    List(Vec<Sexp>),
    /// empty, trivial symbolic-expression
    Empty,
}

/*
impl PartialEq for Sexp {
    fn eq(&self, other: &Sexp) -> bool {
        match *self {
            Sexp::String(ref s) => {
                match *other {
                    Sexp::String(ref s2) => {
                        s == s2
                    },
                    _ => false,
                }
            },
            Sexp::List(ref v) => {
                match *other {
                    Sexp::List(ref v2) => {
                        v == v2
                    },
                    _ => false,
                }
            },
            Sexp::Empty => {
                match *other {
                    Sexp::Empty => true,
                    _ => false,
                }
            }
        }
    }
}
*/

// Following the KiCad file formats specification chapter 4.4 - Identifiers and Strings:
// A string may not contain an actual newline or carriage return,
// but it may use an escape sequence to encode a // newline, such as \n.
// If a string has any of the following conditions, then it must be quoted
// with a leading and trailing double quote
// character, otherwise it is acceptable to not quote the string:
// 1. has one or more of the following 4 separator bytes: ASCII space,
// tab, '(', or ')'.
// 2. has one or more of the following bytes: '%', '{', or '}'.
// 3. has a length of zero bytes, and you need a place holder for the field,
//  then use "".
// 4. includes a byte of '-', and this byte is not in the first position of
// the string.
//

// Joost remark: kicad no longer seems to follow Rule 4.
// so we don't either

/// encode a string according to the guidelines given by Kicad
pub fn encode_string(s: &str) -> String {
    // fn rule_4(s:&str) -> bool {
    // s.contains('-') && s.len() > 1 && s.as_bytes()[0] != 45
    // }
    if s.contains('(') || s.contains(' ') || s.contains(')') || s.contains('\t') ||
       s.contains('{') || s.contains('}') || s.contains('}') || s.contains('%') ||
       s.is_empty()
    // || rule_4(s)
    {
        format!("\"{}\"", s)
    } else {
        String::from(s)
    }
}

impl Sexp {
    /// create an empty symbolic-expression
    pub fn new_empty() -> Sexp {
        Sexp::Empty
    }

    /// create a String type symbolic-expression
    pub fn new_string<T>(s: T) -> Sexp
        where T: fmt::Display
    {
        Sexp::String(format!("{}", s))
    }

    /// create a list type symbolic-expression
    pub fn new_list(v: Vec<Sexp>) -> Sexp {
        Sexp::List(v)
    }

    /// create a list type symbolic-expression where
    /// the first element of the list is a string that indicates
    /// the name, the remainder is filled in via the provided
    /// fill function
    pub fn new_named_list<F>(name: &str, fill: F) -> Sexp
        where F: Fn(&mut Vec<Sexp>)
    {
        let mut v = vec![];
        v.push(Sexp::new_string(name));
        fill(&mut v);
        Sexp::List(v)
    }

    /// create a list type symbolic-expression where
    /// the first element of the list is a string that indicates
    /// the name
    pub fn new_named<T>(name: &str, value: T) -> Sexp
        where T: fmt::Display
    {
        let mut v = vec![];
        v.push(Sexp::new_string(name));
        v.push(Sexp::new_string(value));
        Sexp::List(v)
    }

    /// create a list type symbolic-expression where
    /// the first element of the list is a string that indicates
    /// the name, and the second is another symbolic-expression
    /// created via the IntoSexp trait
    pub fn new_named_sexp<T>(name: &str, value: &T) -> Sexp
        where T: IntoSexp
    {
        let mut v = vec![];
        v.push(Sexp::new_string(name));
        v.push(value.into_sexp());
        Sexp::List(v)
    }

    /// create a symbolic-expression via the IntoSexp trait
    pub fn from<T: IntoSexp>(t: &T) -> Sexp {
        t.into_sexp()
    }

    /// access the symbolic-expression as if it is a List
    pub fn list(&self) -> Result<&Vec<Sexp>> {
        match *self {
            Sexp::List(ref v) => Ok(v),
            _ => str_error(format!("not a list: {}", self)),
        }
    }

    /// access the symbolic-expression as if it is a String
    pub fn string(&self) -> Result<&String> {
        match *self {
            Sexp::String(ref s) => Ok(s),
            _ => str_error(format!("not a string: {}", self)),
        }
    }

    /// access the symbolic-expression as if it is a String
    /// that is a f64
    pub fn f(&self) -> Result<f64> {
        let s = try!(self.string());
        match f64::from_str(s) {
            Ok(f) => Ok(f),
            _ => str_error(format!("Error parsing as float {}", self)),
        }
    }

    /// access the symbolic-expression as if it is a String
    /// that is an i64
    pub fn i(&self) -> Result<i64> {
        let s = try!(self.string());
        match i64::from_str(s) {
            Ok(f) => Ok(f),
            _ => str_error(format!("Error parsing as int {}", self)),
        }
    }

    /// access the symbolic-expression as if it is a List
    /// assuming the first element is a String and return that
    pub fn list_name(&self) -> Result<&String> {
        let l = try!(self.list());
        let l = &l[..];
        let a = try!(l[0].string());
        Ok(a)
    }

    /// access the symbolic-expression as if it is a named List
    /// where the name is provided and returns the remaining elements
    /// after the name as a slice
    pub fn slice_atom(&self, s: &str) -> Result<&[Sexp]> {
        let v = try!(self.list());
        let v2 = &v[..];
        let st = try!(v2[0].string());
        if st != s {
            return str_error(format!("list {} doesn't start with {}, but with {}", self, s, st));
        };
        Ok(&v[1..])
    }

    /// access the symbolic-expression as if it is a named List
    /// with two elements where the name is provided and returns
    /// the next element in the list
    pub fn named_value(&self, s: &str) -> Result<&Sexp> {
        let v = try!(self.list());
        if v.len() != 2 {
            return str_error(format!("list {} is not a named_value", s));
        }
        let l = try!(self.slice_atom(s));
        Ok(&l[0])
    }

    /// as named_value but converted to i64
    pub fn named_value_i(&self, s: &str) -> Result<i64> {
        try!(self.named_value(s)).i()
    }

    /// as named_value but converted to f64
    pub fn named_value_f(&self, s: &str) -> Result<f64> {
        try!(self.named_value(s)).f()
    }

    /// as named_value but converted to String
    pub fn named_value_string(&self, s: &str) -> Result<&String> {
        try!(self.named_value(s)).string()
    }

    /// get the symbolic-expression as a list which starts
    /// with a string that indicates the name and has num more
    /// elements, returns those elements
    pub fn slice_atom_num(&self, s: &str, num: usize) -> Result<&[Sexp]> {
        let v = try!(self.list());
        let v2 = &v[..];
        let st = try!(v2[0].string());
        if st != s {
            return str_error(format!("list doesn't start with {}, but with {}", s, st));
        };
        if v.len() != (num + 1) {
            return str_error(format!("list ({}) doesn't have {} elements but {}",
                                     s,
                                     num,
                                     v.len() - 1));
        }
        Ok(&v[1..])
    }
}

impl fmt::Display for Sexp {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match *self {
            Sexp::String(ref s) => write!(f, "{}", encode_string(s)),
            Sexp::List(ref v) => {
                try!(write!(f, "("));
                let l = v.len();
                for (i, x) in v.iter().enumerate() {
                    if i < l - 1 {
                        try!(write!(f, "{} ", x));
                    } else {
                        try!(write!(f, "{}", x));
                    }
                }
                write!(f, ")")
            }
            Sexp::Empty => Ok(()),
        }
    }
}
