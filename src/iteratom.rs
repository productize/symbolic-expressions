// (c) 2017 Productize SPRL <joost@productize.be>

use std::iter::Peekable;
use std::slice::Iter;

use Sexp;
pub use Result as SResult;

/// convert an `&Sexp` to something
pub trait FromSexp
    where Self: Sized
{
    /// convert from a symbolic-expression to something
    fn from_sexp(&Sexp) -> SResult<Self>;
}

/// convert from a symbolic-expression to something (dispatcher)
pub fn from_sexp<T: FromSexp>(s: &Sexp) -> SResult<T> {
    T::from_sexp(s)
}

/// Atom iterator wrapper
pub struct IterAtom<'a> {
    name: String,
    /// containing iterable (pub for now)
    pub iter: Peekable<Iter<'a, Sexp>>,
}


impl<'a> IterAtom<'a> {
    /// deconstruct a `Sexp` that is a list and starts with 'name'
    pub fn new(s: &'a Sexp, name: &str) -> SResult<IterAtom<'a>> {
        let v = s.list()?;
        let mut i = v.iter();
        let st = match i.next() {
            None => return Err(format!("missing first element {} in list {}", name, s).into()),
            Some(e) => e.string()?,
        };
        if st != name {
            return Err(format!("list {} doesn't start with {}, but with {}", s, name, st).into());
        }
        let i = i.peekable();
        Ok(IterAtom {
            name: name.into(),
            iter: i,
        })
    }

    fn expect<T, F>(&mut self, name: &str, get: F) -> SResult<T>
        where F: Fn(&Sexp) -> SResult<T>
    {
        match self.iter.next() {
            Some(x) => get(x),
            None => return Err(format!("missing {} field in {}", name, self.name).into()),
        }
    }

    /// expect an integer while iterating a `Sexp` list
    pub fn i(&mut self, name: &str) -> SResult<i64> {
        self.expect(name, |x| x.i().map_err(From::from))
    }

    /// expect a float while iterating a `Sexp` list
    pub fn f(&mut self, name: &str) -> SResult<f64> {
        self.expect(name, |x| x.f().map_err(From::from))
    }

    /// expect a String while iterating a `Sexp` list
    pub fn s(&mut self, name: &str) -> SResult<String> {
        self.expect(name, |x| x.s().map_err(From::from))
    }

    /// expect a list contained String while iterating a `Sexp` list
    pub fn s_in_list(&mut self, name: &str) -> SResult<String> {
        self.expect(name,
                    |x| x.named_value_s(name).map_err(From::from))
    }

    /// expect a list contained i64 while iterating a `Sexp` list
    pub fn i_in_list(&mut self, name: &str) -> SResult<i64> {
        self.expect(name, |x| x.named_value_i(name).map_err(From::from))
    }

    /// expect a list contained f64 while iterating a `Sexp` list
    pub fn f_in_list(&mut self, name: &str) -> SResult<f64> {
        self.expect(name, |x| x.named_value_f(name).map_err(From::from))
    }


    /// expect a `Sexp` while iterating a `Sexp` list
    pub fn t<T: FromSexp>(&mut self, name: &str) -> SResult<T> {
        self.expect(name, |x| T::from_sexp(x))
    }

        /// expect a list containing a `Sexp` while iterating a `Sexp` list
    pub fn t_in_list<T: FromSexp>(&mut self, name: &str) -> SResult<T> {
        self.expect(name, |x| T::from_sexp(x.named_value(name)?))
    }

    /// expect remainder of iterator to be a `Vec<T>`
    pub fn vec<T: FromSexp>(&mut self) -> SResult<Vec<T>> {
        let mut res = Vec::new();
        loop {
            match self.iter.next() {
                None => break,
                Some(e) => {
                    let p = from_sexp(e)?;
                    res.push(p);
                }
            }
        }
        Ok(res)
    }

    /// maybe something while iterating a `Sexp` list
    fn maybe<X, F>(&mut self, convert: F) -> Option<X>
        where F: Fn(&Sexp) -> SResult<X>
    {
        let res = match self.iter.peek() {
            None => None,
            Some(s) => {
                match convert(s) {
                    Ok(t) => Some(t),
                    Err(_) => None,
                }
            }
        };
        match res {
            Some(x) => {
                let _ = self.iter.next();
                Some(x)
            }
            x => x,
        }
    }

    /// maybe a `FromSexp` while iterating a `Sexp` list
    pub fn maybe_t<T: FromSexp>(&mut self) -> Option<T> {
        self.maybe(|x| T::from_sexp(x))
    }

    /// maybe a `String` while iterating a `Sexp` list
    pub fn maybe_s(&mut self) -> Option<String> {
        self.maybe(|x| x.s())
    }
    
    /// maybe an `i64` while iterating a `Sexp` list
    pub fn maybe_i(&mut self) -> Option<i64> {
        self.maybe(|x| x.i())
    }

    /// maybe an `f64` while iterating a `Sexp` list
    pub fn maybe_f(&mut self) -> Option<f64> {
        self.maybe(|x| x.f())
    } 

    /// maybe a list containing a `String` while iterating a `Sexp` list
    pub fn maybe_s_in_list(&mut self, name:&str) -> Option<String> {
        self.maybe(|x| x.named_value_s(name))
    }

    /// maybe a list containing an `i64` while iterating a `Sexp` list
    pub fn maybe_i_in_list(&mut self, name:&str) -> Option<i64> {
        self.maybe(|x| x.named_value_i(name))
    }

    /// maybe a list containing an `f64` while iterating a `Sexp` list
    pub fn maybe_f_in_list(&mut self, name:&str) -> Option<f64> {
        self.maybe(|x| x.named_value_f(name))
    }
}
