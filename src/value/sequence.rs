use super::Value;

use crate::error::{ Error, Result };
use serde::de::{ DeserializeSeed, SeqAccess };

pub struct Seq {
    pub seq: Vec<Value>,
}

impl<'de> SeqAccess<'de> for Seq {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where T: DeserializeSeed<'de> {
        // The `Vec` is reversed, so we can pop to get the originally first element
        self.seq
            .pop()
            .map_or(Ok(None), |v| seed.deserialize(v).map(Some))
    }
} 