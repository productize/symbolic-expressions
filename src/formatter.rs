// (c) 2016 Productize SPRL <joost@productize.be>

use std::io;
use std::collections::HashMap;

use Result;
use Sexp;

pub trait Formatter {
    /// Called when serializing a '('.
    fn open<W>(&mut self, writer: &mut W, value:Option<&Sexp>) -> Result<()>
        where W: io::Write;

    /// Called when serializing a ' VAL'.
    fn element<W>(&mut self, writer: &mut W, value:&Sexp) -> Result<()>
        where W: io::Write;

    /// Called when serializing a ')'.
    fn close<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write;
}

pub struct CompactFormatter;

impl Formatter for CompactFormatter {
    fn open<W>(&mut self, writer: &mut W, _value:Option<&Sexp>) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b"(").map_err(From::from)
    }
    fn element<W>(&mut self, writer: &mut W, _value:&Sexp) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b" ").map_err(From::from)
    }
    
    fn close<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b")").map_err(From::from)
    }
}

pub struct RulesFormatter {
    indent:Vec<u8>,
    indent_before:HashMap<&'static str, i64>,
}

impl Default for RulesFormatter {
    fn default() -> Self {
        let idb = HashMap::new();
        RulesFormatter {
            indent:vec![b' ',b' '], // two spaces
            indent_before:idb,
        }
    }
}
    

// TODO: get rid of kicad specifics in RulesFormatter

impl RulesFormatter {

    pub fn new_kicad() -> RulesFormatter {
        let mut rf = RulesFormatter::default();
        rf.indent_before.insert("layer", 1);
        rf.indent_before.insert("desc", 1);
        rf.indent_before.insert("fp_text", 1);
        rf.indent_before.insert("fp_poly", 1);
        rf.indent_before.insert("fp_line", 1);
        rf.indent_before.insert("pad", 1);
        rf.indent_before.insert("general", 1);
        rf
    }
}

impl Formatter for RulesFormatter {
    fn open<W>(&mut self, writer: &mut W, value:Option<&Sexp>) -> Result<()>
        where W: io::Write
    {
        // if first element is string and it has an indent setting
        if let Some(ref sexp) = value {
            if let Sexp::String(ref s) = **sexp {
                let s:&str = &s;
                if let Some(&i) = self.indent_before.get(s) {
                    try!(writer.write_all(b"\n"));
                    for _ in 0..i {
                    try!(writer.write_all(&self.indent));
                    }
                }
            }
        }
        writer.write_all(b"(").map_err(From::from)
    }
    
    fn element<W>(&mut self, writer: &mut W, value:&Sexp) -> Result<()>
        where W: io::Write
    {
        // if containing value is a list and it has indent_before
        // don't put the space
        if let Sexp::List(ref l) = *value {
            if !l.is_empty() {
                if let Sexp::String(ref s) = l[0] {
                    let s2:&str = &s; // why needed?
                    if self.indent_before.contains_key(s2) {
                        return Ok(())
                    }
                }
            }
        }
        writer.write_all(b" ").map_err(From::from)
    }
    
    fn close<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b")").map_err(From::from)
    }
}
