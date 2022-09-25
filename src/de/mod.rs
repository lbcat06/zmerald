mod value;
mod map;

#[cfg(test)]
mod tests;

mod id;
pub use id::IdDeserializer;

mod tag;
pub use tag::TagDeserializer;

use crate::error::{ Error, SpannedError, Result, SpannedResult };
use crate::parse::{ AnyNum, Bytes, ParsedStr };
use serde::de::{ self, DeserializeSeed, Deserializer as SerdeError, Visitor };
use std::{ borrow::Cow, io, str };

pub fn from_reader<R, T>(mut rdr: R) -> SpannedResult<T> where R: io::Read, T: de::DeserializeOwned {
    let mut bytes = Vec::new();
    rdr.read_to_end(&mut bytes)?;

    from_bytes(&bytes)
}

pub fn from_str<'a, T>(s: &'a str) -> SpannedResult<T> where T: de::Deserialize<'a> {
    from_bytes(s.as_bytes())
}

pub fn from_bytes<'a, T>(s: &'a [u8]) -> SpannedResult<T> where T: de::Deserialize<'a> {
    from_bytes_seed(s, std::marker::PhantomData)
}

pub fn from_bytes_seed<'a, S, T>(s: &'a [u8], seed: S) -> SpannedResult<T>
where S: de::DeserializeSeed<'a, Value = T> {
    let mut deserializer = Deserializer::from_bytes(s)?;
    let value = seed.deserialize(&mut deserializer).map_err(|e| deserializer.span_error(e))?;
    deserializer.end().map_err(|e| deserializer.span_error(e))?;
    Ok(value)
}

pub struct Deserializer<'de> {
    bytes: Bytes<'de>,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> SpannedResult<Self> {
        Self::from_bytes(input.as_bytes())
    }

    pub fn from_bytes(input: &'de [u8]) -> SpannedResult<Self> {
        let deserializer = Deserializer {
            bytes: Bytes::new(input)?,
        };

        Ok(deserializer)
    }

    pub fn remainder(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.bytes.bytes())
    }

    pub fn span_error(&self, code: Error) -> SpannedError {
        self.bytes.span_error(code)
    }
}

impl<'de> Deserializer<'de> {
    /// Check if the remaining bytes are whitespace only, otherwise return an error.
    pub fn end(&mut self) -> Result<()> {
        self.bytes.skip_ws()?;

        if self.bytes.bytes().is_empty() {
            Ok(())
        } else {
            Err(Error::TrailingCharacters)
        }
    }

    fn handle_other_structs<V>(&mut self, visitor: V) -> Result<V::Value>
    where V: Visitor<'de> {
        let mut bytes = self.bytes;

        if bytes.consume("(") {
            bytes.skip_ws()?;

            if bytes.check_tuple_struct()? {
                self.deserialize_tuple(0, visitor)
            } else {
               Err(Error::ExpectedTupleStruct) 
            }
        } else {
            visitor.visit_unit()
        }
    }
}

// impl<'de, 'a> de::Deserialize<'de> for &'a mut Deserializer<'de> {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: de::Deserializer<'de> {
//         struct Visitor;

//         impl<'de, 'a> de::Visitor<'de> for &'a mut Deserializer<'de> {
//             type Value = &'a mut Deserializer<'de>;

//             fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//                 formatter.write_str("stuff")
//             }
//         }

//         fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
//             todo!();

//             let mut res: Map = Map::new();
//             while let Some(entry) = map.next_entry()? {
//                 res.insert(entry.0, entry.1);
//             }

//             Ok(Value::Map(res))
//         }

//         todo!()
//     }
// }

// need to edit the visit_map() of the below Visitor, how does one do that?
impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if self.bytes.consume_ident("true") {
            return visitor.visit_bool(true);
        } else if self.bytes.consume_ident("false") {
            return visitor.visit_bool(false);
        } else if self.bytes.check_ident("Some") {
            return self.deserialize_option(visitor);
        } else if self.bytes.consume_ident("None") {
            return visitor.visit_none();
        } else if self.bytes.consume("()") {
            return visitor.visit_unit();
        } else if self.bytes.consume_ident("inf") {
            return visitor.visit_f64(std::f64::INFINITY);
        } else if self.bytes.consume_ident("-inf") {
            return visitor.visit_f64(std::f64::NEG_INFINITY);
        } else if self.bytes.consume_ident("NaN") {
            return visitor.visit_f64(std::f64::NAN);
        }

        // `identifier` does not change state if it fails
        if self.bytes.identifier().ok().is_some() {
            self.bytes.skip_ws()?;
            return self.handle_other_structs(visitor);
        }

        match self.bytes.peek_or_eof()? {
            b'0'..=b'9' | b'+' | b'-' => {
                match self.bytes.any_num()? {
                    AnyNum::F32(x) => visitor.visit_f32(x),
                    AnyNum::F64(x) => visitor.visit_f64(x),
                    AnyNum::I8(x) => visitor.visit_i8(x),
                    AnyNum::U8(x) => visitor.visit_u8(x),
                    AnyNum::I16(x) => visitor.visit_i16(x),
                    AnyNum::U16(x) => visitor.visit_u16(x),
                    AnyNum::I32(x) => visitor.visit_i32(x),
                    AnyNum::U32(x) => visitor.visit_u32(x),
                    AnyNum::I64(x) => visitor.visit_i64(x),
                    AnyNum::U64(x) => visitor.visit_u64(x),
                }
            },

            b'{' | b'<' => self.deserialize_map(visitor),
            b'"' | b'r' => self.deserialize_string(visitor),
            b'(' => self.handle_other_structs(visitor),
            b'[' => self.deserialize_seq(visitor),
            b'.' => self.deserialize_f64(visitor),
            b'\'' => self.deserialize_char(visitor),
            other => Err(Error::UnexpectedByte(other as char))
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_bool(self.bytes.bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_i8(self.bytes.signed_integer()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_i16(self.bytes.signed_integer()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_i32(self.bytes.signed_integer()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_i64(self.bytes.signed_integer()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_u8(self.bytes.unsigned_integer()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_u16(self.bytes.unsigned_integer()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_u32(self.bytes.unsigned_integer()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_u64(self.bytes.unsigned_integer()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_f32(self.bytes.float()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_f64(self.bytes.float()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_char(self.bytes.char()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        match self.bytes.string()? {
            ParsedStr::Allocated(s) => visitor.visit_string(s),
            ParsedStr::Slice(s) => visitor.visit_borrowed_str(s),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let res = {
            let string = self.bytes.string()?;
            let base64_str = match string {
                ParsedStr::Allocated(ref s) => s.as_str(),
                ParsedStr::Slice(s) => s,
            };

            base64::decode(base64_str)
        };

        match res {
            Ok(byte_buf) => visitor.visit_byte_buf(byte_buf),
            Err(err) => Err(Error::Base64Error(err)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if self.bytes.consume("None") {
            visitor.visit_none()
        } else if self.bytes.consume("Some") && { self.bytes.skip_ws()?; self.bytes.consume("(") } {
            self.bytes.skip_ws()?;
            let v = visitor.visit_some(&mut *self)?;
            self.bytes.skip_ws()?;

            if self.bytes.consume(")") {
                Ok(v)
            } else {
                Err(Error::ExpectedOptionEnd)
            }
        } else {
            visitor.visit_some(&mut *self)
        }
    }

    // In Serde, unit means an anonymous value containing no data
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if self.bytes.consume("{}") {
            visitor.visit_unit()
        } else {
            Err(Error::ExpectedUnit)
        }
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if self.bytes.consume_struct_name(name)? {
            visitor.visit_unit()
        } else {
            self.deserialize_unit(visitor)
        }
    }

    // HMM
    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        self.bytes.consume_struct_name(name)?;
        self.bytes.skip_ws()?;

        if self.bytes.consume("(") {
            self.bytes.skip_ws()?;
            let value = visitor.visit_newtype_struct(&mut *self)?;
            self.bytes.comma()?;

            if self.bytes.consume(")") {
                Ok(value)
            } else {
                Err(Error::ExpectedStructEnd)
            }
        } else if name.is_empty() {
            Err(Error::ExpectedStruct)
        } else {
            Err(Error::ExpectedNamedStruct(name))
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if self.bytes.consume("[") {
            let value = visitor.visit_seq(CommaSeparated::new(b']', self))?;
            self.bytes.comma()?;

            if self.bytes.consume("]") {
                Ok(value)
            } else {
                Err(Error::ExpectedArrayEnd)
            }
        } else {
            Err(Error::ExpectedArray)
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if self.bytes.consume("(") {
            let value = visitor.visit_seq(CommaSeparated::new(b')', self))?;
            self.bytes.comma()?;

            if self.bytes.consume(")") {
                Ok(value)
            } else {
                Err(Error::ExpectedArrayEnd)
            }
        } else {
            Err(Error::ExpectedArray)
        }
    }

    fn deserialize_tuple_struct<V>(self, name: &'static str, len: usize, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        self.bytes.consume_struct_name(name)?;
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if self.bytes.consume("{") {
            let value = visitor.visit_map(CommaSeparated::new(b'}', self))?;
            self.bytes.comma()?;

            if self.bytes.consume("}") {
                Ok(value)
            } else {
                Err(Error::ExpectedMapEnd)
            }
        } else {
            Err(Error::ExpectedMap)
        }
    }

    fn deserialize_struct<V>(self, name: &'static str, _fields: &'static [&'static str], visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        self.bytes.consume_struct_name(name)?;
        self.bytes.skip_ws()?;

        if self.bytes.consume("{") {
            let value = visitor.visit_map(CommaSeparated::new(b'}', self))?;
            self.bytes.comma()?;

            if self.bytes.consume("}") {
                // duplicate error occurs after this is returned
                Ok(value)
            } else {
                Err(Error::ExpectedStructEnd)
            }
        } else if name.is_empty() {
            Err(Error::ExpectedStruct)
        } else {
            Err(Error::ExpectedNamedStruct(name))
        }
    }

    fn deserialize_enum<V>(self,_name: &'static str, _variants: &'static [&'static str], visitor: V) -> Result<V::Value>
    where V: Visitor<'de> {
        visitor.visit_enum(Enum::new(self))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where V: Visitor<'de> {
        visitor.visit_str(
            str::from_utf8(self.bytes.identifier()?).map_err(Error::from)?,
        )
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }
}

struct CommaSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    terminator: u8,
    had_comma: bool,
}

impl<'a, 'de> CommaSeparated<'a, 'de> {
    fn new(terminator: u8, de: &'a mut Deserializer<'de>) -> Self {
        CommaSeparated {
            de,
            terminator,
            had_comma: true,
        }
    }

    fn has_element(&mut self) -> Result<bool> {
        self.de.bytes.skip_ws()?;

        match (self.had_comma, self.de.bytes.peek_or_eof()? != self.terminator) {
            // Trailing comma, maybe has a next element
            (true, has_element) => Ok(has_element),
            // No trailing comma but terminator
            (false, false) => Ok(false),
            // No trailing comma or terminator
            (false, true) => Err(Error::ExpectedComma), 
        }
    }
}

impl<'de, 'a> de::SeqAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>> where T: DeserializeSeed<'de> {
        if self.has_element()? {
            let res = seed.deserialize(&mut *self.de)?;
            self.had_comma = self.de.bytes.comma()?;

            Ok(Some(res))
        } else {
            Ok(None)
        }
    }
}

impl<'de, 'a> de::MapAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>> where K: DeserializeSeed<'de> {
        if self.has_element()? {
            if self.de.bytes.consume("<") {
                return seed.deserialize(&mut *self.de).map(Some);
            } else if self.terminator == b')' {
                return seed.deserialize(&mut IdDeserializer::new(&mut *self.de)).map(Some);
            } 

            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value> where V: DeserializeSeed<'de> {
        self.de.bytes.skip_ws()?;

        if self.de.bytes.consume(":") || self.de.bytes.consume(">") {
            self.de.bytes.skip_ws()?;
            let res = seed.deserialize(&mut TagDeserializer::new(&mut *self.de))?;
            self.had_comma = self.de.bytes.comma()?;

            Ok(res)
        } else {
            Err(Error::ExpectedMapSeparator)
        }
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

impl<'de, 'a> de::EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where V: DeserializeSeed<'de> {
        self.de.bytes.skip_ws()?;

        let value = seed.deserialize(&mut *self.de)?;

        Ok((value, self))
    }
}

impl<'de, 'a> de::VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where T: DeserializeSeed<'de> {
        self.de.bytes.skip_ws()?;

        if self.de.bytes.consume("(") {
            self.de.bytes.skip_ws()?;
            let val = seed.deserialize(&mut *self.de)?;
            self.de.bytes.comma()?;

            if self.de.bytes.consume(")") {
                Ok(val)
            } else {
                Err(Error::ExpectedStructEnd)
            }
        } else {
            Err(Error::ExpectedStruct)
        }
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where V: Visitor<'de> {
        self.de.bytes.skip_ws()?;
        self.de.deserialize_tuple(len, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where V: Visitor<'de> {
        self.de.bytes.skip_ws()?;
        self.de.deserialize_struct("", fields, visitor)
    }
}