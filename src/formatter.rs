use std::io;

use Result;
use Sexp;

pub trait Formatter {
    /// Called when serializing a '('.
    fn open<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write;

    /// Called when serializing a ' '.
    fn element<W>(&mut self, writer: &mut W, first:bool, value:&Sexp) -> Result<()>
        where W: io::Write;

    /// Called when serializing a ')'.
    fn close<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write;
}

pub struct CompactFormatter;

impl Formatter for CompactFormatter {
    fn open<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b"(").map_err(From::from)
    }
    fn element<W>(&mut self, writer: &mut W, first:bool, value:&Sexp) -> Result<()>
        where W: io::Write
    {
        if first {
            Ok(())
        } else {
            writer.write_all(b" ").map_err(From::from)
        }
    }
    
    fn close<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b")").map_err(From::from)
    }
}

pub struct KicadFormatter;

impl Formatter for KicadFormatter {
    fn open<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b"(").map_err(From::from)
    }
    
    fn element<W>(&mut self, writer: &mut W, first:bool, value:&Sexp) -> Result<()>
        where W: io::Write
    {
        if first {
            Ok(())
        } else {
            writer.write_all(b" ").map_err(From::from)
        }
    }
    
    fn close<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        writer.write_all(b")").map_err(From::from)
    }
}
