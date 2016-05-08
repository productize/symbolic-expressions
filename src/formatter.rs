use std::io;

use Result;

pub trait Formatter {
    /// Called when serializing a '('.
    fn open<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write;

    /// Called when serializing a ' '.
    fn space<W>(&mut self, writer: &mut W, first:bool) -> Result<()>
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
    fn space<W>(&mut self, writer: &mut W, first:bool) -> Result<()>
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
