// (c) 2016 Joost Yervante Damad <joost@productize.be>

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
}

impl de::Deserializer for Deserializer {
    type Error = Error;

    /// not used
    fn deserialize<V>(&mut self, mut visitor: V)
                      -> Result<V::Value>
        where V: de::Visitor
    {
        match self.exp {
            Sexp::String(_) => self.deserialize_string(visitor),
            _ => Err(Error::Decoder(format!("expecting specific deserializer to be called for {}", self.exp)))
        }
    }

    /// deserialize any string in a symbolic expression
    fn deserialize_string<V>(&mut self, mut visitor: V) -> Result<V::Value> 
        where V: de::Visitor {
        match self.exp {
            Sexp::String(ref s) => visitor.visit_string(s.clone()),
            _ => Err(Error::Decoder(format!("expecting string got {}", self.exp)))
        }
    }

    /// the empty symbolic-expression
    fn deserialize_unit<V>(&mut self, mut visitor: V) -> Result<V::Value> 
        where V: de::Visitor {
        match self.exp {
            Sexp::Empty => visitor.visit_unit(),
            _ => Err(Error::Decoder(format!("expecting unit got {}", self.exp)))
        }
    }

    /// a symbolic expression of the form (name value1 value2 ...)
    fn deserialize_tuple_struct<V>(&mut self,
                               name: &'static str,
                               len: usize,
                               mut visitor: V)
                               -> Result<V::Value>
        where V: de::Visitor {
        match self.exp {
            Sexp::List(ref v) => {
                if v.len() != len+1 {
                    return Err(Error::Decoder(format!("expecting {} elements for tuple struct in {}", len, self.exp)))
                }
                match v[0] {
                    Sexp::String(ref name2) => {
                        if name != name2.to_lowercase().as_str() {
                            return Err(Error::Decoder(format!("expecting name {} got {} in {}", name, name2, self.exp)))
                        }
                        visitor.visit_seq(SeqVisitor::new(v, true))
                    }
                    _ => Err(Error::Decoder(format!("expecting string as first element of list got {}", self.exp)))
                }
            },
            _ => Err(Error::Decoder(format!("expecting list got {}", self.exp)))
        }
    }

    fn deserialize_struct<V>(&mut self,
                         name: &'static str,
                         fields: &'static [&'static str],
                         mut visitor: V)
                         -> Result<V::Value>
        where V: de::Visitor {
        match self.exp {
            Sexp::List(ref v) => {
                if v.len() != fields.len()+1 {
                    return Err(Error::Decoder(format!("expecting {} elements for struct in {}", fields.len(), self.exp)))
                }
                match v[0] {
                    Sexp::String(ref name2) => {
                        let name2 = name2.to_lowercase();
                        let name = name.to_lowercase();
                        if name != name2 {
                            return Err(Error::Decoder(format!("expecting name {} got {} in {}", name, name2, self.exp)))
                        }
                        visitor.visit_map(StructVisitor::new(v))
                    }
                    _ => Err(Error::Decoder(format!("expecting string as first element of list got {}", self.exp)))
                }
            },
            _ => Err(Error::Decoder(format!("expecting list got {}", self.exp))),
        }
    }

    fn deserialize_seq<V>(&mut self, mut visitor: V) -> Result<V::Value> 
        where V: de::Visitor {
        match self.exp {
            Sexp::List(ref v) => visitor.visit_seq(SeqVisitor::new(v, false)),
            _ => Err(Error::Decoder(format!("expecting seq got {}", self.exp))),
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
        seq_fixed_size bytes map unit_struct 
        struct_field tuple ignored_any option enum
    }
}

struct SeqVisitor<'a> {
    seq: &'a Vec<Sexp>,
    i:usize,
}

impl<'a> SeqVisitor<'a> {
    fn new(seq: &'a Vec<Sexp>, skip:bool) -> Self {
        let i = if skip { 1 } else { 0 };
        SeqVisitor {
            seq: seq,
            i: i,
        }
    }
}

impl<'a> de::SeqVisitor for SeqVisitor<'a> {
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>>
        where T: de::Deserialize,
    {
        if self.i >= self.seq.len() {
            return Ok(None)
        }
        let ref t = self.seq[self.i];
        self.i += 1;
        de::Deserialize::deserialize(&mut Deserializer::new(t.clone())).map(Some)
    }

    fn end(&mut self) -> Result<()> {
        Ok(())
    }
}

struct StructVisitor<'a> {
    seq: &'a Vec<Sexp>,
    i:usize,
    value: Option<Sexp>,
}

impl<'a> StructVisitor<'a> {
    fn new(seq:&'a Vec<Sexp>) -> Self {
        StructVisitor {
            seq: seq,
            i:1,
            value: None,
        }
    }
}

impl<'a> de::MapVisitor for StructVisitor<'a> {
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>>
        where K: de::Deserialize,
    {
        if self.i >= self.seq.len() {
            return Ok(None)
        }
        let ref exp = self.seq[self.i];
        self.i += 1;
        match *exp {
            Sexp::List(ref v) => {
                if v.len() != 2 {
                    return Err(Error::Decoder("can't decode as map 1".into()))
                }
                if let Ok(k) = v[0].string() {
                    self.value = Some(v[1].clone());
                    de::Deserialize::deserialize(&mut Deserializer::new(v[0].clone())).map(Some)
                } else {
                    return Err(Error::Decoder(format!("key is not a string: {}", v[0])))
                }
            },
            _ => Err(Error::Decoder("can't decode as map 3".into()))
        }
    }

    fn visit_value<V>(&mut self) -> Result<V>
        where V: de::Deserialize,
    {
        if let Some(ref v) = self.value {
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
