// (c) 2015-2016 Productize SPRL <joost@productize.be>

//! symbolic-expressions parsing and generating library

#![warn(missing_docs)]

#![cfg_attr(feature = "serde_derive", feature(proc_macro))]
#[cfg(feature = "serde_derive")]
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde;

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

/// serde symbolic-expression decoding code: symbolic-expression -> rust
pub mod decode;

pub mod data;

#[cfg(test)]
mod test;
