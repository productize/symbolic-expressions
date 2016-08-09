// (c) 2015-2016 Productize SPRL <joost@productize.be>

//! symbolic-expressions parsing and generating library

#![warn(missing_docs)]

#[macro_use]
extern crate nom;

pub use error::*;
pub use sexp::*;

mod error;
mod formatter;
mod sexp;

/// symbolic-expression parser code
pub mod parser;

/// symbolic-expression parser code expirimental improved version
pub mod parser2;

/// symbolic-expression serialization code
pub mod ser;

pub use formatter::Rules;
pub use formatter::Formatter;

#[cfg(test)]
mod test;
