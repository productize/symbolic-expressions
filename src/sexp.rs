// (c) 2016-2017 Productize SPRL <joost@productize.be>

use std::cmp::max;
use std::fmt;
use std::mem;

use error::SexpError;
/// like Into trait but works from a ref avoiding consumption or expensive clone
pub trait IntoSexp {
    /// convert self into a Sexp
    fn into_sexp(&self) -> Sexp;
}

impl Into<Sexp> for Vec<Sexp> {
    fn into(self: Vec<Sexp>) -> Sexp {
        Sexp::List(self)
    }
}

impl Into<Sexp> for String {
    fn into(self: String) -> Sexp {
        Sexp::Symbol(self)
    }
}

impl<'a> From<&'a str> for Sexp {
    fn from(t: &str) -> Sexp {
        Sexp::Symbol(t.into())
    }
}

impl<'a> From<&'a String> for Sexp {
    fn from(t: &String) -> Sexp {
        Sexp::Symbol(t.clone())
    }
}

impl Into<Sexp> for i64 {
    fn into(self: i64) -> Sexp {
        Sexp::Symbol(format!("{}", self))
    }
}

impl Into<Sexp> for f64 {
    fn into(self: f64) -> Sexp {
        Sexp::Symbol(format!("{}", self))
    }
}

impl<'a> From<(&'a str, Sexp)> for Sexp {
    fn from(kv: (&str, Sexp)) -> Sexp {
        let (name, value) = kv;
        let mut v = vec![];
        v.push(name.into());
        v.push(value);
        v.into()
    }
}

impl<'a, T: fmt::Display> From<(&'a str, &'a T)> for Sexp {
    fn from(kv: (&str, &T)) -> Sexp {
        let (name, value) = kv;
        let mut v = vec![];
        v.push(name.into());
        v.push(format!("{}", value).into());
        v.into()
    }
}

/// a symbolic-expression structure
/// Can be a string or a list or nothing
///
/// `String` shape: "hello"
/// `Symbol` shape: hello
/// `List`   shape: (...)
/// `Empty   shape:
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Sexp {
    /// a string
    String(String),
    /// a symbol
    Symbol(String),
    /// list symbolic-expression
    List(Vec<Sexp>),
    /// empty, trivial symbolic-expression
    #[default]
    Empty,
}

impl Sexp {
    /// create an empty symbolic-expression
    #[deprecated(since = "4.0.0", note = "please use `Sexp::default()` instead")]
    pub fn new_empty() -> Sexp {
        Sexp::Empty
    }

    /// create a String type symbolic-expression
    #[deprecated(since = "4.0.3", note = "please use `.into()` instead")]
    pub fn new_string<T>(s: T) -> Sexp
    where
        T: fmt::Display,
    {
        format!("{}", s).into()
    }

    /// create a list type symbolic-expression
    #[deprecated(since = "4.0.3", note = "please use `.into()` instead")]
    pub fn new_list(v: Vec<Sexp>) -> Sexp {
        Sexp::List(v)
    }

    /// create an empty list type symbolic-expression
    pub fn start(name: &str) -> Sexp {
        let mut v = vec![];
        v.push(Sexp::Symbol(name.into()));
        Sexp::List(v)
    }

    /// push an element in a list
    pub fn push<T: Into<Sexp>>(&mut self, element: T) {
        match *self {
            Sexp::List(ref mut v) => v.push(element.into()),
            _ => panic!("Only use push on lists!"),
        }
    }

    /// create a list type symbolic-expression where
    /// the first element of the list is a string that indicates
    /// the name, the remainder is filled in via the provided
    /// fill function
    ///
    /// shape: (name ...)
    pub fn new_named_list<F>(name: &str, fill: F) -> Sexp
    where
        F: Fn(&mut Vec<Sexp>),
    {
        let mut v = vec![];
        v.push(name.into());
        fill(&mut v);
        Sexp::List(v)
    }

    /// create a list type symbolic-expression where
    /// the first element of the list is a string that indicates
    /// the name
    #[deprecated(since = "4.0.3", note = "please use `.into()` instead")]
    pub fn new_named<T>(name: &str, value: T) -> Sexp
    where
        T: fmt::Display,
    {
        let mut v = vec![];
        v.push(name.into());
        v.push(format!("{}", value).into());
        v.into()
    }

    /// create a list type symbolic-expression where
    /// the first element of the list is a string that indicates
    /// the name, and the second is another symbolic-expression
    /// created via the IntoSexp trait
    #[deprecated(since = "4.0.5", note = "please use `.into()` on a tuple instead")]
    pub fn new_named_sexp<T>(name: &str, value: &T) -> Sexp
    where
        T: IntoSexp,
    {
        let mut v = vec![];
        v.push(name.into());
        v.push(value.into_sexp());
        Sexp::List(v)
    }

    /// if the expression is a list, extract the `Vec<Sexp>`
    /// from it and swap it with Empty
    pub fn take_list(&mut self) -> Result<Vec<Sexp>, SexpError> {
        let mut e = Sexp::Empty;
        mem::swap(&mut e, self);
        match e {
            Sexp::List(v) => Ok(v),
            _ => Err(format!("Not a list: {}", e).into()),
        }
    }

    /// if the expression is a `String`, take it out and swap it with Empty
    pub fn take_string(&mut self) -> Result<String, SexpError> {
        let mut e = Sexp::Empty;
        mem::swap(&mut e, self);
        match e {
            Sexp::Symbol(s) => Ok(s),
            _ => Err(format!("Not a string: {}", e).into()),
        }
    }

    /// create a symbolic-expression via the `IntoSexp` trait
    pub fn from<T: IntoSexp>(t: &T) -> Sexp {
        t.into_sexp()
    }

    /// access the symbolic-expression as if it is a List
    pub fn list(&self) -> Result<&Vec<Sexp>, SexpError> {
        match *self {
            Sexp::List(ref v) => Ok(v),
            _ => Err(format!("not a list: {}", self).into()),
        }
    }

    /// access the symbolic-expression as if it is an `&String`
    pub fn string(&self) -> Result<&String, SexpError> {
        match *self {
            Sexp::Symbol(ref s) => Ok(s),
            _ => Err(format!("not a string: {}", self).into()),
        }
    }

    /// access the symbolic-expression as if it is a `String`
    pub fn s(&self) -> Result<String, SexpError> {
        match *self {
            Sexp::Symbol(ref s) => Ok(s.clone()),
            _ => Err(format!("not a string: {}", self).into()),
        }
    }

    /// is this expression a string
    pub fn is_string(&self) -> bool {
        match *self {
            Sexp::Symbol(_) => true,
            _ => false,
        }
    }

    /// is this expression a list
    pub fn is_list(&self) -> bool {
        match *self {
            Sexp::List(_) => true,
            _ => false,
        }
    }

    /// access the symbolic-expression as if it is a String
    /// that is a f64
    pub fn f(&self) -> Result<f64, SexpError> {
        let s = self.string()?;
        let f = s.parse()?;
        Ok(f)
    }

    /// access the symbolic-expression as if it is a String
    /// that is an i64
    pub fn i(&self) -> Result<i64, SexpError> {
        let s = self.string()?;
        let i = s.parse()?;
        Ok(i)
    }

    /// access the symbolic-expression as if it is a List
    /// assuming the first element is a String and return that
    pub fn list_name(&self) -> Result<&String, SexpError> {
        let l = self.list()?;
        let l = &l[..];
        let a = l[0].string()?;
        Ok(a)
    }

    /// access the symbolic-expression as if it is a named List
    /// where the name is provided and returns the remaining elements
    /// after the name as a slice
    #[deprecated(since = "4.1.4", note = "please use `iteratom::IterAtom::new` instead")]
    pub fn slice_atom(&self, s: &str) -> Result<&[Sexp], SexpError> {
        let v = self.list()?;
        let v2 = &v[..];
        let st = v2[0].string()?;
        if st != s {
            return Err(format!("list {} doesn't start with {}, but with {}", self, s, st).into());
        };
        Ok(&v[1..])
    }

    fn slice_atom_int(&self, s: &str) -> Result<&[Sexp], SexpError> {
        let v = self.list()?;
        let v2 = &v[..];
        let st = v2[0].string()?;
        if st != s {
            return Err(format!("list {} doesn't start with {}, but with {}", self, s, st).into());
        };
        Ok(&v[1..])
    }

    /// access the symbolic-expression as if it is a named List
    /// with two elements where the name is provided and returns
    /// the next element in the list
    pub fn named_value(&self, s: &str) -> Result<&Sexp, SexpError> {
        let v = self.list()?;
        if v.len() != 2 {
            return Err(format!("list {} is not a named_value", s).into());
        }
        let l = self.slice_atom_int(s)?;
        Ok(&l[0])
    }

    /// as named_value but converted to i64
    pub fn named_value_i(&self, s: &str) -> Result<i64, SexpError> {
        self.named_value(s)?.i()
    }

    /// as named_value but converted to f64
    pub fn named_value_f(&self, s: &str) -> Result<f64, SexpError> {
        self.named_value(s)?.f()
    }

    /// as named_value but converted to `&String`
    pub fn named_value_string(&self, s: &str) -> Result<&String, SexpError> {
        self.named_value(s)?.string()
    }

    /// as named_value but converted to `String`
    pub fn named_value_s(&self, s: &str) -> Result<String, SexpError> {
        Ok(self.named_value(s)?.string()?.clone())
    }

    /// get the symbolic-expression as a list which starts
    /// with a string that indicates the name and has num more
    /// elements, returns those elements
    #[deprecated(since = "4.1.4", note = "please use `iteratom::IterAtom::new` instead")]
    pub fn slice_atom_num(&self, s: &str, num: usize) -> Result<&[Sexp], SexpError> {
        let v = self.list()?;
        let v2 = &v[..];
        let st = v2[0].string()?;
        if st != s {
            return Err(format!("list doesn't start with {}, but with {}", s, st).into());
        };
        if v.len() != (num + 1) {
            return Err(format!(
                "list ({}) doesn't have {} elements but {}",
                s,
                num,
                v.len() - 1
            )
            .into());
        }
        Ok(&v[1..])
    }

    fn how_many_childen_inline(&self) -> usize {
        let minimum = 1;
        let mut res = 0;
        if let Sexp::List(children) = self {
            for child in children {
                match child {
                    Sexp::List(l) => {
                        if l.len() == 1 {
                            res += 1;
                        }
                    }
                    _ => res += 1,
                }
            }
        }

        max(minimum, res)
    }

    /// Produces a pretty-printed s-expression
    pub fn pretty(&self) -> String {
        // use a stack so we don't run out of stack space
        // level, s-exp, and if we have already recurred
        let mut stack = vec![(0, self, false)];
        let mut return_stack = vec![];

        let get_spacing = |level: usize| -> String {
            let mut res = String::new();
            for _ in 0..level {
                res.push_str("  ");
            }
            res
        };

        while let Some((start_level, current, has_recur)) = stack.pop() {
            if !has_recur {
                stack.push((start_level, current, true));
                match current {
                    Sexp::Symbol(_) => (),
                    Sexp::String(_) => (),
                    Sexp::List(lists) => {
                        let how_many_inline = current.how_many_childen_inline();
                        for (i, l) in lists.iter().enumerate() {
                            let new_level = if i < how_many_inline {
                                0
                            } else {
                                start_level + 1
                            };
                            stack.push((new_level, l, false))
                        }
                    }
                    Sexp::Empty => (),
                }
            } else {
                match current {
                    Sexp::String(_) => {
                        return_stack.push(format!("{}\"{}\"", get_spacing(start_level), current))
                    }
                    Sexp::Symbol(_) => {
                        return_stack.push(format!("{}{}", get_spacing(start_level), current))
                    }
                    Sexp::List(children) => {
                        let mut res = String::new();
                        res.push_str(&get_spacing(start_level));
                        res.push('(');

                        let how_many_inline = current.how_many_childen_inline();
                        for (i, _child) in children.iter().enumerate() {
                            match i {
                                0 => (),
                                a if a < how_many_inline => res.push(' '),
                                _ => {
                                    res.push('\n');
                                }
                            }
                            res.push_str(&return_stack.pop().unwrap());
                        }
                        res.push(')');
                        return_stack.push(res);
                    }
                    Sexp::Empty => return_stack.push("".to_string()),
                }
            }
        }

        return_stack.pop().unwrap()
    }
}

impl fmt::Display for Sexp {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Sexp::String(ref s) => write!(f, "\"{}\"", s),
            Sexp::Symbol(ref s) => write!(f, "{}", s),
            Sexp::List(ref v) => {
                write!(f, "(")?;
                let l = v.len();
                for (i, x) in v.iter().enumerate() {
                    if i < l - 1 {
                        write!(f, "{} ", x)?;
                    } else {
                        write!(f, "{}", x)?;
                    }
                }
                write!(f, ")")
            }
            Sexp::Empty => Ok(()),
        }
    }
}
