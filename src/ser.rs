use std::io;

use formatter::*;

use Sexp;
use Result;

pub struct Serializer<W, F=CompactFormatter> {
    writer: W,
    formatter: F,
}

// dispatches only based on Formatter
impl<W> Serializer<W>
    where W: io::Write,
{
    pub fn new(writer: W) -> Self {
        Serializer::with_formatter(writer, CompactFormatter)
    }
}

impl<W> Serializer<W, RulesFormatter>
    where W: io::Write,
{
    /* TODO
    fn new_rules(writer: W) -> Self {
        Serializer::with_formatter(writer, RulesFormatter::new())
    }
     */
    
    pub fn new_kicad(writer: W) -> Self {
        Serializer::with_formatter(writer, RulesFormatter::new_kicad())
    }
}

impl<W, F> Serializer<W, F>
    where W: io::Write,
          F: Formatter,
{
    fn with_formatter(writer: W, formatter: F) -> Self {
        Serializer {
            writer: writer,
            formatter: formatter,
        }
    }

    fn serialize_str(&mut self, value:&str) -> Result<()> {
        if value.contains('(') || value.contains(' ') || value.is_empty() {
            write!(&mut self.writer, "\"{}\"", value).map_err(From::from)
        } else {
            write!(&mut self.writer, "{}", value).map_err(From::from)
        }
    }

    pub fn serialize(&mut self, value:&Sexp) -> Result<()> {
        match *value {
            Sexp::String(ref s) => {
                self.serialize_str(s)
            },
            Sexp::List(ref list) => {
                let mut first = true;
                if list.is_empty() {
                    try!(self.formatter.open(&mut self.writer, None));
                } else {                   
                    for v in list {
                        if first {
                            try!(self.formatter.open(&mut self.writer, Some(v)));
                            try!(self.serialize(v));
                            first = false;
                        } else {
                            try!(self.formatter.element(&mut self.writer, v));
                            try!(self.serialize(v));
                        }
                    }
                }
                self.formatter.close(&mut self.writer)
            },
            Sexp::Empty => Ok(()),
        }
        
    }
}
