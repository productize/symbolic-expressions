// (c) 2015-2016 Productize SPRL <joost@productize.be>

#![cfg_attr(feature = "use_clippy", feature(plugin))]
#![cfg_attr(feature = "use_clippy", plugin(clippy))]

#[macro_use]
extern crate nom;

pub use error::*;
pub use sexp::*;

mod error;
mod formatter;
mod sexp;

pub mod parser;
pub mod ser;

pub use formatter::Rules;
pub use formatter::Formatter;

#[cfg(test)]
mod test;
