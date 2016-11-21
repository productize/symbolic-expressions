// (c) 2016 Joost Yervante Damad <joost@productize.be>

use std::error;
use std::fmt;

use serde;
use serde::de;
use Sexp;

// loosely based on toml-rs

/// Description for errors which can occur while decoding a type.
#[derive(PartialEq, Debug)]
pub struct DecodeError {
    /// Field that this error applies to.
    pub field: Option<String>,
    /// The type of error which occurred while decoding,
    pub kind: DecodeErrorKind,
}

impl de::Error for DecodeError {
    fn custom<T: Into<String>>(msg: T) -> DecodeError {
        DecodeError {
            field: None,
            kind: DecodeErrorKind::CustomError(msg.into()),
        }
    }
    fn end_of_stream() -> DecodeError {
        DecodeError { field: None, kind: DecodeErrorKind::EndOfStream }
    }
    fn missing_field(name: &'static str) -> DecodeError {
        DecodeError {
            field: Some(name.to_string()),
            kind: DecodeErrorKind::ExpectedField(None),
        }
    }
    fn unknown_field(name: &str) -> DecodeError {
        DecodeError {
            field: Some(name.to_string()),
            kind: DecodeErrorKind::UnknownField,
        }
    }
    fn invalid_type(ty: de::Type) -> Self {
        DecodeError {
            field: None,
            kind: DecodeErrorKind::InvalidType(match ty {
                de::Type::Bool => "bool",
                de::Type::Usize |
                de::Type::U8 |
                de::Type::U16 |
                de::Type::U32 |
                de::Type::U64 |
                de::Type::Isize |
                de::Type::I8 |
                de::Type::I16 |
                de::Type::I32 |
                de::Type::I64 => "integer",
                de::Type::F32 |
                de::Type::F64 => "float",
                de::Type::Char |
                de::Type::Str |
                de::Type::String => "string",
                de::Type::Seq => "array",
                de::Type::Struct |
                de::Type::Map => "table",
                de::Type::Unit => "Unit",
                de::Type::Option => "Option",
                de::Type::UnitStruct => "UnitStruct",
                de::Type::NewtypeStruct => "NewtypeStruct",
                de::Type::TupleStruct => "TupleStruct",
                de::Type::FieldName => "FieldName",
                de::Type::Tuple => "Tuple",
                de::Type::Enum => "Enum",
                de::Type::VariantName => "VariantName",
                de::Type::StructVariant => "StructVariant",
                de::Type::TupleVariant => "TupleVariant",
                de::Type::UnitVariant => "UnitVariant",
                de::Type::Bytes => "Bytes",
            })
        }
    }
}

impl error::Error for DecodeError {
    fn description(&self) -> &str {
        match self.kind {
            ApplicationError(ref s) => &**s,
            ExpectedField(..) => "expected a field",
            UnknownField => "found an unknown field",
            ExpectedType(..) => "expected a type",
            ExpectedMapKey(..) => "expected a map key",
            ExpectedMapElement(..) => "expected a map element",
            NoEnumVariants => "no enum variants to decode to",
            NilTooLong => "nonzero length string representing nil",
            SyntaxError => "syntax error",
            EndOfStream => "end of stream",
            InvalidType(..) => "invalid type",
            CustomError(..) => "custom error",
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum DecodeErrorKind {
    /// An error flagged by the application, e.g. value out of range
    ApplicationError(String),
    /// A field was expected, but none was found.
    ExpectedField(/* type */ Option<&'static str>),
    /// A field was found, but it was not an expected one.
    UnknownField,
    /// A field was found, but it had the wrong type.
    ExpectedType(/* expected */ &'static str, /* found */ &'static str),
    /// The nth map key was expected, but none was found.
    ExpectedMapKey(usize),
    /// The nth map element was expected, but none was found.
    ExpectedMapElement(usize),
    /// An enum decoding was requested, but no variants were supplied
    NoEnumVariants,
    /// The unit type was being decoded, but a non-zero length string was found
    NilTooLong,
    /// There was an error with the syntactical structure of the TOML.
    SyntaxError,
    /// A custom error was generated when decoding.
    CustomError(String),
    /// The end of the TOML input was reached too soon
    EndOfStream,
    /// Produced by serde ...
    InvalidType(&'static str),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(match self.kind {
            ApplicationError(ref err) => {
                write!(f, "{}", err)
            }
            ExpectedField(expected_type) => {
                match expected_type {
                    Some("table") => write!(f, "expected a section"),
                    Some(e) => write!(f, "expected a value of type `{}`", e),
                    None => write!(f, "expected a value"),
                }
            }
            UnknownField => write!(f, "unknown field"),
            ExpectedType(expected, found) => {
                fn humanize(s: &str) -> String {
                    if s == "section" {
                        "a section".to_string()
                    } else {
                        format!("a value of type `{}`", s)
                    }
                }
                write!(f, "expected {}, but found {}",
                       humanize(expected),
                       humanize(found))
            }
            ExpectedMapKey(idx) => {
                write!(f, "expected at least {} keys", idx + 1)
            }
            ExpectedMapElement(idx) => {
                write!(f, "expected at least {} elements", idx + 1)
            }
            NoEnumVariants => {
                write!(f, "expected an enum variant to decode to")
            }
            NilTooLong => {
                write!(f, "expected 0-length string")
            }
            SyntaxError => {
                write!(f, "syntax error")
            }
            EndOfStream => {
                write!(f, "end of stream")
            }
            InvalidType(s) => {
                write!(f, "invalid type: {}", s)
            }
            CustomError(ref s) => {
                write!(f, "custom error: {}", s)
            }
        });
        match self.field {
            Some(ref s) => {
                write!(f, " for the key `{}`", s)
            }
            None => Ok(())
        }
    }
}

use self::DecodeErrorKind::*;

pub fn decode<T:serde::Deserialize>(exp: Sexp) -> Option<T> {
    serde::Deserialize::deserialize(&mut Decoder::new(exp)).ok()
}

pub struct Decoder {
    /// The expression value left over after decoding. This can be used to inspect
    /// whether fields were decoded or not.
    pub exp: Option<Sexp>,
    cur_field: Option<String>,
}

impl Decoder {
    /// Creates a new decoder, consuming the Sexp value to decode.
    ///
    /// This decoder can be passed to the `Decodable` methods or driven
    /// manually.
    pub fn new(exp: Sexp) -> Decoder {
        Decoder::new_empty(Some(exp), None)
    }

    fn sub_decoder(&self, sexp: Option<Sexp>, field: &str) -> Decoder {
        let cur_field = if field.is_empty() {
            self.cur_field.clone()
        } else {
            match self.cur_field {
                None => Some(field.to_string()),
                Some(ref s) => Some(format!("{}.{}", s, field))
            }
        };
        Decoder::new_empty(sexp, cur_field)
    }

    fn new_empty(exp: Option<Sexp>, cur_field: Option<String>) -> Decoder {
        Decoder {
            exp: exp,
            cur_field: cur_field,
        }
    }

    fn err(&self, kind: DecodeErrorKind) -> DecodeError {
        DecodeError {
            field: self.cur_field.clone(),
            kind: kind,
        }
    }

    fn mismatch(&self, expected: &'static str,
                found: &Option<Sexp>) -> DecodeError{
        match *found {
            Some(ref val) => self.err(ExpectedType(expected, &format!("{}", val))),
            None => self.err(ExpectedField(Some(expected))),
        }
    }
}

impl de::Deserializer for Decoder {
    type Error = DecodeError;

    fn deserialize<V>(&mut self, mut visitor: V)
                      -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        match self.exp.take() {
            Some(Sexp::String(s)) => visitor.visit_string(s),
            Some(Sexp::Empty) => visitor.visit_unit(),
            Some(Sexp::List(a)) => {
                let len = a.len();
                let iter = a.into_iter();
                visitor.visit_seq(SeqDeserializer::new(iter, len, &mut self.exp))
            }
            None => Err(self.err(DecodeErrorKind::EndOfStream)),
        }
    }

    fn deserialize_bool<V>(&mut self, mut visitor: V)
                           -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        match self.exp.take() {
            ref found@Some(Sexp::String(b)) => {
                match b.as_str() {
                    "true" | "True" => visitor.visit_bool(true),
                    "false" | "False" => visitor.visit_bool(false),
                    _ => Err(self.mismatch("bool", found)),
                }
            },
            ref found => Err(self.mismatch("bool", found)),
        }
    }

    fn deserialize_i8<V>(&mut self, visitor: V)
                         -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V>(&mut self, visitor: V)
                          -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V>(&mut self, visitor: V)
                          -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i64<V>(&mut self, mut visitor: V)
                          -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        match self.exp.take() {
            ref found@Some(Sexp::String(s)) => {
                match s.parse::<i64>() {
                    Ok(f) => visitor.visit_i64(f),
                    Err(_) => Err(self.mismatch("integer", found)),
                }
            },
            ref found => Err(self.mismatch("integer", found)),
        }
    }

    fn deserialize_isize<V>(&mut self, visitor: V)
                            -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u8<V>(&mut self, visitor: V)
                         -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u16<V>(&mut self, visitor: V)
                          -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u32<V>(&mut self, visitor: V)
                          -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u64<V>(&mut self, visitor: V)
                          -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_usize<V>(&mut self, visitor: V)
                            -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_f32<V>(&mut self, visitor: V)
                          -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V>(&mut self, mut visitor: V)
                          -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        match self.exp.take() {
            ref found@Some(Sexp::String(s)) => {
                match s.parse::<f64>() {
                    Ok(f) => visitor.visit_f64(f),
                    Err(_) => Err(self.mismatch("float", found)),
                }
            },
            ref found => Err(self.mismatch("float", found)),
        }
    }
    
    fn deserialize_str<V>(&mut self, mut visitor: V)
                          -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        match self.exp.take() {
            Some(Sexp::String(s)) => visitor.visit_string(s),
            ref found => Err(self.mismatch("string", found)),
        }
    }

    fn deserialize_string<V>(&mut self, visitor: V)
                             -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_char<V>(&mut self, mut visitor: V)
                           -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        match self.exp.take() {
            Some(Sexp::String(ref s)) if s.chars().count() == 1 => {
                visitor.visit_char(s.chars().next().unwrap())
            }
            ref found => return Err(self.mismatch("string", found)),
        }
    }
    
    fn deserialize_option<V>(&mut self, mut visitor: V)
                             -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        if self.exp.is_none() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }
        
    fn deserialize_seq<V>(&mut self, mut visitor: V)
                          -> Result<V::Value, DecodeError>
        where V: de::Visitor,
    {
        if self.exp.is_none() {
            let iter = None::<i32>.into_iter();
            visitor.visit_seq(de::value::SeqDeserializer::new(iter, 0))
        } else {
            self.deserialize(visitor)
        }
    }

    fn deserialize_map<V>(&mut self, mut visitor: V)
                          -> Result<V::Value, DecodeError>
        where V: de::Visitor,
    {
        match self.exp.take() {
            Some(Sexp::List(t)) => {
                visitor.visit_map(MapVisitor {
                    iter: t.into_iter(),
                    de: self,
                    key: None,
                    value: None,
                })
            }
            ref found => Err(self.mismatch("table", found)),
        }
    }

    fn deserialize_enum<V>(&mut self,
                           _enum: &str,
                           variants: &[&str],
                           mut visitor: V) -> Result<V::Value, DecodeError>
        where V: de::EnumVisitor,
    {
        Err(self.err(CustomError("enum type unsupported".into())))
    }
    
 fn deserialize_ignored_any<V>(&mut self, visitor: V)
                                  -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        use serde::de::value::ValueDeserializer;
        let mut d = <() as ValueDeserializer<Self::Error>>::into_deserializer(());
        d.deserialize(visitor)
    }

    fn deserialize_bytes<V>(&mut self, visitor: V)
                            -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_seq_fixed_size<V>(&mut self, _len: usize, visitor: V)
                                     -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        self.deserialize_seq(visitor)
    }
    
    fn deserialize_newtype_struct<V>(&mut self, _name: &'static str, visitor: V)
                                     -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(&mut self,
                                   _name: &'static str,
                                   _len: usize,
                                   visitor: V)
                                   -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_struct<V>(&mut self,
                             _name: &'static str,
                             _fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_tuple<V>(&mut self,
                            _len: usize,
                            visitor: V)
                            -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        self.deserialize_seq(visitor)
    }
    
    fn deserialize_unit<V>(&mut self, visitor:V) -> Result<V::Value, Self::Error>
        where V: de::Visitor {
        self.deserialize(visitor)
    }
    
    fn deserialize_unit_struct<V>(&mut self, _name: &'static str, visitor:V) -> Result<V::Value, Self::Error>
        where V: de::Visitor {
        self.deserialize(visitor)
    }

    fn deserialize_struct_field<V>(&mut self, visitor:V) -> Result<V::Value, Self::Error>
        where V: de::Visitor {
        self.deserialize(visitor)
    }
}
