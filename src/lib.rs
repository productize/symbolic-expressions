// (c) 2015-2016 Productize SPRL <joost@productize.be>

//! symbolic-expressions parsing and generating library

#![warn(missing_docs)]

pub use error::*;
pub use sexp::*;

mod error;
mod formatter;
mod sexp;

/// symbolic-expression parser code: data -> symbolic-expression
pub mod parser;

/// symbolic-expression serialization code: symbolic-expression -> data
pub mod ser;

pub use formatter::Rules;
pub use formatter::Formatter;

#[cfg(test)]
mod test;
