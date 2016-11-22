// (c) 2016 Joost Yervante Damad <joost@productize.be>

use std::error;
use std::fmt;
use std::slice;
use std::result;

use serde;
use serde::de;

use Sexp;

// loosely based on serde-yaml

// TODO: get rid of some of the clones

use error::{Result,Error};

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

     fn deserialize_sexp_list<V:de::Visitor>(&mut self, mut visitor: V, v:Vec<Sexp>) -> Result<V::Value> {
        match v[0] {
            // if first element is a string, we consider it a struct
            Sexp::String(s) => {
                // if all other elements are tuples with the first
                // being a name, consider it a struct
                let mut is_struct = true;
                for e in &v[1..] {
                    match *e {
                        Sexp::List(v2) => {
                            if v2.len() != 2 {
                                is_struct = false
                            }
                        },
                        _ => {
                            is_struct = false
                        },
                    }
                }
                if is_struct {
                    return visitor.visit_map(MapVisitor::new(v))
                }
                // else consider it a tuple struct; skip first element
                visitor.visit_seq(SeqVisitor::new(v, true))
                
            },
            // if first element is not a string, we consider it a plain sequence
            _ => {
                visitor.visit_seq(SeqVisitor::new(v, false))
            },
        }
    }
}

impl de::Deserializer for Deserializer {
    type Error = Error;

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
                    self.deserialize_sexp_list(visitor, v)
                }
            }
        }
    }

    /// deserialize any string in a symbolic expression
    fn deserialize_string<V>(&mut self, visitor: V) -> Result<V::Value> 
        where V: de::Visitor {
        match self.exp {
            Sexp::String(s) => visitor.visit_string(s),
            _ => Err(Error::Decode(format!("expecting string got {}", self.exp)))
        }
    }

    /// the  symbolic-expression
    fn deserialize_unit<V>(&mut self, visitor: V) -> Result<V::Value> 
        where V: de::Visitor {
        match self.exp {
            Sexp::Empty => visitor.visit_unit(),
            _ => Err(Error::Decode(format!("expecting unit got {}", self.exp)))
        }
    }

    /// a symbolic expression of the form (name value1 value2 ...)
    fn deserialize_tuple_struct<V>(&mut self,
                               name: &'static str,
                               len: usize,
                               visitor: V)
                               -> Result<V::Value, Self::Error> 
        where V: de::Visitor {
        match self.exp {
            Sexp::List(v) => {
                if v.len() != len+1 {
                    return Err(Error::Decode(format!("expecting {} elements for tuple struct in {}", len, self.exp)))
                }
                match v[0] {
                    Sexp::String(name2) => {
                        if name != name2.as_str() {
                            return Err(Error::Decode(format!("expecting name {} got {} in {}", name, name2, self.exp)))
                        }
                        visitor.visit_seq(SeqVisitor::new(v, true))
                    }
                    _ => Err(Error::Decode(format!("expecting string as first element of list got {}", self.exp)))
                }
            }
            _ => Err(Error::Decode(format!("expecting list got {}", self.exp)))
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
        bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str
        unit seq seq_fixed_size bytes map unit_struct tuple_struct struct
        struct_field tuple ignored_any option enum
    }
}

struct SeqVisitor<'a> {
    iter: slice::Iter<'a, Sexp>,
}

impl<'a> SeqVisitor<'a> {
    fn new(seq: Vec<Sexp>, skip:bool) -> Self {
        let mut iter = seq.iter();
        if skip {
            iter.next();
        }
        SeqVisitor {
            iter: iter,
        }
    }
}

impl<'a> de::SeqVisitor for SeqVisitor<'a> {
    type Error = Error;

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

struct MapVisitor<'a> {
    iter: slice::Iter<'a, Sexp>,
    /// Value associated with the most recently visited key.
    v: Option<&'a Sexp>,
}

impl<'a> MapVisitor<'a> {
    fn new(seq: Vec<Sexp>) -> Self {
        let iter = seq.iter();
        iter.next(); // skip name element
        MapVisitor {
            iter: iter,
            v: None,
        }
    }
}

impl<'a> de::MapVisitor for MapVisitor<'a> {
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>>
        where K: de::Deserialize,
    {
        match self.iter.next() {
            None => Ok(None),
            Some(exp) => {
                match exp {
                    Sexp::List(v) => {
                        if v.len() != 2 {
                            return Err(Error::Decode("can't decode as map 1".into()))
                        }
                        if let Ok(k) = v[0].string() {
                            self.v = v[1];
                            de::Deserialize::deserialize(&mut Deserializer::new(k)).map(Some)
                        } else {
                            return Err(Error::Decode("can't decode as map 2".into()))
                        }
                        self.v = Some(v);
                        de::Deserialize::deserialize(&mut Deserializer::new(k)).map(Some)
                    },
                    _ => Err(Error::Decode("can't decode as map 3".into()))
            }
        }
        }
    }

    fn visit_value<V>(&mut self) -> Result<V>
        where V: de::Deserialize,
    {
        if let Some(v) = self.v {
            de::Deserialize::deserialize(&mut Deserializer::new(v.clone()))
        } else {
            panic!("must call visit_key before visit_value")
        }
    }

    fn end(&mut self) -> Result<()> {
        Ok(())
    }

    fn missing_field<V>(&mut self, field: &'static str) -> Result<V>
        where V: de::Deserialize,
    {
        struct MissingFieldDeserializer(&'static str);

        impl de::Deserializer for MissingFieldDeserializer {
            type Error = Error;

            fn deserialize<V>(&mut self, _visitor: V) -> Result<V::Value>
                where V: de::Visitor,
            {
                Err(de::Error::missing_field(self.0))
            }

            fn deserialize_option<V>(
                &mut self,
                mut visitor: V
            ) -> Result<V::Value>
                where V: de::Visitor,
            {
                visitor.visit_none()
            }

            forward_to_deserialize!{
                bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str
                string unit seq seq_fixed_size bytes map unit_struct
                newtype_struct tuple_struct struct struct_field tuple enum
                ignored_any
            }
        }

        let mut de = MissingFieldDeserializer(field);
        Ok(try!(de::Deserialize::deserialize(&mut de)))
    }
}
