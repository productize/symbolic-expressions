// (c) 2016 Joost Yervante Damad <joost@productize.be>

use std::error;
use std::fmt;
use std::slice;
use std::result;

use serde;
use serde::de;

use Sexp;

// loosely based on serde-yaml

// TODO
#[derive(Debug)]
struct DecodeError {
    msg:String,
}

impl error::Error for DecodeError {
    fn description(&self) -> &str {
        self.msg.as_str()
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DecodeError {}", self.msg)
    }
}

impl de::Error for DecodeError {
    fn custom<T: Into<String>>(msg: T) -> Self {
        DecodeError { msg:msg.into() }
    }

    fn end_of_stream() -> Self {
        DecodeError { msg:"end_of_stream".into() }
    }
}

pub type Result<T> = result::Result<T, DecodeError>;

pub fn decode<T:serde::Deserialize>(exp: Sexp) -> Result<T> {
    serde::Deserialize::deserialize(&mut Deserializer::new(exp))
}

pub struct Deserializer {
    pub exp: Sexp,
}

impl Deserializer {
    pub fn new(exp: Sexp) -> Deserializer {
        Deserializer { exp: exp }
    }
}

impl de::Deserializer for Deserializer {
    type Error = DecodeError;

    fn deserialize<V>(&mut self, mut visitor: V)
                      -> Result<V::Value>
        where V: de::Visitor
    {
        match self.exp {
            Sexp::Empty => visitor.visit_unit(),
            Sexp::String(s) => visitor.visit_string(s),
            Sexp::List(v) => {
                if v.is_empty() {
                    visitor.visit_unit()
                } else {
                    match v[0] {
                        Sexp::String(s) => {
                            visitor.visit_map(MapVisitor::new(v))
                        },
                        _ => {
                            visitor.visit_seq(SeqVisitor::new(v))
                        },
                    }
                }
            }
        }
    }

    /// Parses a newtype struct as the underlying value.
    fn deserialize_newtype_struct<V>(
        &mut self,
        _name: &str,
        mut visitor: V
    ) -> Result<V::Value>
        where V: de::Visitor,
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize!{
        bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str string
        unit seq seq_fixed_size bytes map unit_struct tuple_struct struct
        struct_field tuple ignored_any option enum
    }
}

struct SeqVisitor<'a> {
    /// Iterator over the YAML array being visited.
    iter: slice::Iter<'a, Sexp>,
}

impl<'a> SeqVisitor<'a> {
    fn new(seq: Vec<Sexp>) -> Self {
        SeqVisitor {
            iter: seq.iter(),
        }
    }
}

impl<'a> de::SeqVisitor for SeqVisitor<'a> {
    type Error = DecodeError;

    fn visit<T>(&mut self) -> Result<Option<T>>
        where T: de::Deserialize,
    {
        match self.iter.next() {
            None => Ok(None),
            Some(t) => {
                de::Deserialize::deserialize(&mut Deserializer::new(t.clone())).map(Some)
            }
        }
    }

    fn end(&mut self) -> Result<()> {
        Ok(())
    }
}
