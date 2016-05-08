use std::io;
use std::collections::HashMap;

use Result;
use Sexp;

pub trait Formatter {
    /// Called when serializing a '(string'.
    fn open<W>(&mut self, writer: &mut W, value:&str) -> Result<()>
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
    fn open<W>(&mut self, writer: &mut W, value:&str) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b"(").map_err(From::from)
    }
    fn element<W>(&mut self, writer: &mut W, value:&Sexp) -> Result<()>
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

// TODO: get rid of kicad specifics in RulesFormatter

impl RulesFormatter {

    pub fn new() -> RulesFormatter {
        let mut idb = HashMap::new();
        RulesFormatter {
            indent:vec![b' ',b' '], // two spaces
            indent_before:idb,
        }
    }
    
    pub fn new_kicad() -> RulesFormatter {
        let mut rf = RulesFormatter::new();
        rf.indent_before.insert("layer", 1);
        rf.indent_before.insert("desc", 1);
        rf.indent_before.insert("fp_text", 1);
        rf.indent_before.insert("fp_poly", 1);
        rf.indent_before.insert("fp_line", 1);
        rf.indent_before.insert("pad", 1);
        rf
    }
}
    
    
impl Formatter for RulesFormatter {
    fn open<W>(&mut self, writer: &mut W, value:&str) -> Result<()>
        where W: io::Write
    {
        match self.indent_before.get(value) {
            Some(&i) => {
                try!(writer.write_all(b"\n"));
                for _ in 0..i {
                    try!(writer.write_all(&self.indent));
                }
            },
            None => (),
        }
        writer.write_all(b"(").map_err(From::from)
    }
    
    fn element<W>(&mut self, writer: &mut W, value:&Sexp) -> Result<()>
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
