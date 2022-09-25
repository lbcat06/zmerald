mod map;
pub use map::{ Map, MapAccessor };

mod arithmetic;
pub use arithmetic::Number;

mod sequence;
pub use sequence::Seq;

use serde::de::{ DeserializeOwned, Deserializer, Visitor };
use serde::forward_to_deserialize_any;

use crate::error::{ Error, Result };
use std::cmp::Eq;
use std::hash::Hash;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Value {
    Bool(bool),
    Char(char),
    Map(Map),
    Number(Number),
    Option(Option<Box<Value>>),
    String(String),
    Seq(Vec<Value>),
    Unit,
}

impl Value {
    pub fn into_rust<T>(self) -> Result<T> where T: DeserializeOwned {
        T::deserialize(self)
    }
}

impl<'de> Deserializer<'de> for Value {
    type Error = Error;

    forward_to_deserialize_any! {
        bool f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where V: Visitor<'de> {
        match self {
            Value::Bool(b) => visitor.visit_bool(b),
            Value::Char(c) => visitor.visit_char(c),
            Value::Map(m) => visitor.visit_map(MapAccessor {
                keys: m.keys().cloned().rev().collect(),
                values: m.values().cloned().rev().collect(),
            }),
            Value::Number(Number::Float(ref f)) => visitor.visit_f64(f.get()),
            Value::Number(Number::Integer(i)) => visitor.visit_i64(i),
            Value::Option(Some(o)) => visitor.visit_some(*o),
            Value::Option(None) => visitor.visit_none(),
            Value::String(s) => visitor.visit_string(s),
            Value::Seq(mut seq) => {
                seq.reverse();
                visitor.visit_seq(Seq { seq })
            }
            Value::Unit => visitor.visit_unit(),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where V: Visitor<'de> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where V: Visitor<'de> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where V: Visitor<'de> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where V: Visitor<'de> {
        match self {
            Value::Number(Number::Integer(i)) => visitor.visit_i64(i),
            v => Err(Error::Message(format!("Expected a number, got {:?}", v))),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where V: Visitor<'de> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where V: Visitor<'de> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where V: Visitor<'de> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where V: Visitor<'de> {
        match self {
            Value::Number(Number::Integer(i)) => visitor.visit_u64(i as u64),
            v => Err(Error::Message(format!("Expected a number, got {:?}", v))),
        }
    }
}

//COLOUUUUUUUUUUUURS YOUPI 