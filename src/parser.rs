use std::str::FromStr;
use std::str;

use nom;

use Sexp;
use str_error;
use Result;

pub fn parse_str(sexp: &str) -> Result<Sexp> {
    if sexp.is_empty() {
        return Ok(Sexp::new_empty())
    }
    match parse_sexp(&sexp.as_bytes()[..]) {
        nom::IResult::Done(_, c) => Ok(c),
        nom::IResult::Error(err) => {
            match err {
                nom::Err::Position(kind,p) => 
                    str_error(format!("parse error: {:?} |{}|", kind, str::from_utf8(p).unwrap())),
                _ => str_error("parse error".to_string())
            }
        },
        nom::IResult::Incomplete(x) => str_error(format!("incomplete: {:?}", x)),
    }
}

named!(parse_qstring<String>,
       map_res!(
           map_res!(
               delimited!(char!('\"'), is_not!("\""), char!('\"')),
               str::from_utf8),
           FromStr::from_str)
       );

named!(parse_bare_string<String>,
       map_res!(
           map_res!(
               is_not!(b")( \r\n"),
               str::from_utf8),
           FromStr::from_str)
       );

named!(parse_string<Sexp>,
       map!(alt!(parse_qstring | parse_bare_string), |x| Sexp::String(x))
       );

named!(parse_list<Sexp>,
       chain!(
           char!('(') ~
               v: many0!(parse_sexp) ~
               _space: opt!(nom::multispace) ~ // sometimes there is space after a closing bracket, this would not be caught by parse_sexp
               char!(')'),
           || Sexp::List(v) )
       );

// TODO: consider lines with just spaces and a nl as also nl
named!(line_ending<usize>,
       chain!(
           opt!(nom::space) ~
               c: opt!(is_a!(b"\r\n"))
               , || match c { None => 0, Some(ref x) => x.len(), }
               )
       );

named!(parse_sexp<Sexp>,
           chain!(
               _indent: opt!(nom::space) ~
                   sexp: alt!(parse_list | parse_string) ~
                   _nl: line_ending
                   ,
               || {
                   sexp
               })
       );
