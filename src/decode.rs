// (c) 2016 Joost Yervante Damad <joost@productize.be>

use serde;
use serde::de;
use std::mem;

use Sexp;

use error::{Result, Error};

// loosely based on serde-yaml, toml-rs, serde_json

/// decode a symbolic expression to a rust expression using serde
pub fn decode<T: serde::Deserialize>(exp: Sexp) -> Result<T> {
    serde::Deserialize::deserialize(&mut Deserializer::new(exp))
}

struct Deserializer {
    pub exp: Sexp,
}

impl Deserializer {
    pub fn new(exp: Sexp) -> Deserializer {
        Deserializer { exp: exp }
    }
}

impl de::Deserializer for Deserializer {
    type Error = Error;

    /// called when we call deserialize below for a nested part
    fn deserialize<V>(&mut self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        if self.exp.is_string() {
            return self.deserialize_string(visitor)
        }
        if self.exp.is_list() {
            let v = try!(self.exp.take_list());
            let name = unsafe {
                let s = try!(v[0].string());
                let ret = mem::transmute(&s as &str);
                mem::forget(s);
                ret
            };
            let len = v.len() - 1;
            self.exp = Sexp::List(v);
            return self.deserialize_tuple_struct(name, len, visitor)
        }
        Err(Error::Decoder(format!("expecting specific deserializer to be called for {}", self.exp)))
    }

    /// deserialize any string in a symbolic expression
    fn deserialize_string<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_string(try!(self.exp.take_string()))
    }

    /// the empty symbolic-expression
    fn deserialize_unit<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        match self.exp {
            Sexp::Empty => visitor.visit_unit(),
            _ => Err(Error::Decoder(format!("expecting unit got {}", self.exp))),
        }
    }

    /// a symbolic expression of the form (name value1 value2 ...)
    fn deserialize_tuple_struct<V>(&mut self,
                                   name: &'static str,
                                   len: usize,
                                   mut visitor: V)
                                   -> Result<V::Value>
        where V: de::Visitor
    {
        let v = try!(self.exp.take_list());
        if v.len() != len + 1 {
            return Err(Error::Decoder(format!("expecting {} elements for tuple struct \
                                               in {}",
                                              len,
                                              self.exp)));
        }
        let name2 = try!(v[0].string()).to_lowercase();
        let name = name.to_lowercase();
        if name != name2 {
            return Err(Error::Decoder(format!("expecting name {} got {} in {}",
                                              name,
                                              name2,
                                              self.exp)));
        }
        visitor.visit_seq(SeqVisitor::new(v, true))
    }

    fn deserialize_struct<V>(&mut self,
                             name: &'static str,
                             _fields: &'static [&'static str],
                             mut visitor: V)
                             -> Result<V::Value>
        where V: de::Visitor
    {
        let v = try!(self.exp.take_list());
        if v.len() < 1 {
            return Err(Error::Decoder(format!("missing struct name {} in {:?}",
                                              name,
                                              v)));
        }
        let name2 = try!(v[0].string()).to_lowercase();
        let name = name.to_lowercase();
        if name != name2 {
            return Err(Error::Decoder(format!("expecting name {} got {} in {}",
                                              name,
                                              name2,
                                              self.exp)));
        }
        visitor.visit_map(StructVisitor::new(v))
    }

    fn deserialize_seq<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        let v = try!(self.exp.take_list());
        visitor.visit_seq(SeqVisitor::new(v, false))
    }

    /// Parses a newtype struct as the underlying value.
    fn deserialize_newtype_struct<V>(&mut self, _name: &str, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_option<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_some(self)
    }

    forward_to_deserialize!{
        bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str
        seq_fixed_size bytes map unit_struct 
        struct_field tuple ignored_any enum
    }
}

struct SeqVisitor {
    seq: Vec<Sexp>,
    i: usize,
}

impl SeqVisitor {
    fn new(seq: Vec<Sexp>, skip: bool) -> Self {
        let i = if skip {
            1
        } else {
            0
        };
        SeqVisitor { seq: seq, i: i }
    }
}

impl de::SeqVisitor for SeqVisitor {
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>>
        where T: de::Deserialize
    {
        if self.i >= self.seq.len() {
            return Ok(None);
        }
        let mut t = Sexp::Empty;
        mem::swap(&mut t, &mut self.seq[self.i]);
        self.i += 1;
        de::Deserialize::deserialize(&mut Deserializer::new(t)).map(Some)
    }

    fn end(&mut self) -> Result<()> {
        Ok(())
    }
}

struct StructVisitor {
    seq: Vec<Sexp>,
    i: usize,
    value: Option<Sexp>,
}

impl<'a> StructVisitor {
    fn new(seq: Vec<Sexp>) -> Self {
        StructVisitor {
            seq: seq,
            i: 1,
            value: None,
        }
    }
}

impl de::MapVisitor for StructVisitor {
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>>
        where K: de::Deserialize
    {
        if self.i >= self.seq.len() {
            return Ok(None);
        }
        let mut exp = Sexp::Empty;
        mem::swap(&mut exp, &mut self.seq[self.i]);
        self.i += 1;
        let mut v = try!(exp.take_list());
        if v.len() < 2 {
            return Err(Error::Decoder(format!("can't decode as map: {:?}", v)));
        }
        if v[0].is_string() {
            if v.len() == 2 {
                let mut value = Sexp::Empty;
                mem::swap(&mut value, &mut v[1]);
                self.value = Some(value);
                let mut key = Sexp::Empty;
                mem::swap(&mut key, &mut v[0]);
                de::Deserialize::deserialize(&mut Deserializer::new(key)).map(Some)
            } else {
                // deserialize whole element, which could be a tuple struct
                let key = v[0].clone();
                self.value = Some(Sexp::List(v));
                de::Deserialize::deserialize(&mut Deserializer::new(key)).map(Some)
            }
        } else {
            return Err(Error::Decoder(format!("key is not a string: {}", v[0])));
        }
    }

    fn visit_value<V>(&mut self) -> Result<V>
        where V: de::Deserialize
    {
        let vo = self.value.take();
        match vo {
            Some(v) => de::Deserialize::deserialize(&mut Deserializer::new(v)),
            None => Err(Error::Decoder(format!("missing value!"))),
        }
    }

    fn end(&mut self) -> Result<()> {
        Ok(())
    }

    fn missing_field<V>(&mut self, field: &'static str) -> Result<V>
        where V: de::Deserialize
    {
        struct MissingFieldDeserializer(&'static str);

        impl de::Deserializer for MissingFieldDeserializer {
            type Error = Error;

            fn deserialize<V>(&mut self, _visitor: V) -> Result<V::Value>
                where V: de::Visitor
            {
                Err(de::Error::missing_field(self.0))
            }

            fn deserialize_option<V>(&mut self, mut visitor: V) -> Result<V::Value>
                where V: de::Visitor
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
