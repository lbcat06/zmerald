use std::collections::HashMap;
use std::hash::{ Hash, BuildHasher };

// HashMap<K  , V  <I>  >
// HashMap<u32, Vec<u32>>

pub trait DuplicateAggregates<K, V, I> {
    fn new(size_hint: Option<usize>) -> Self;
    fn insert(&mut self, key: K, value: I);
}

impl<K, V, S, I> DuplicateAggregates<K, V, I> for HashMap<K, V, S>
where K: Eq + Hash, V: Default + IntoIterator<Item = I> + Extend<I>, S: BuildHasher + Default {
    #[inline]
    fn new(size_hint: Option<usize>) -> Self {
        match size_hint {
            Some(size) => Self::with_capacity_and_hasher(size, S::default()),
            None => Self::with_hasher(S::default()),
        }
    }

    #[inline]
    fn insert(&mut self, key: K, value: I) {
        self.entry(key).or_default().extend(std::iter::once(value))
    }
}

use std::fmt;
use serde::{ Deserialize, Serialize, Deserializer, Serializer };
use serde::de::{ Visitor, MapAccess };
use std::marker::PhantomData;

pub fn duplicates_aggregate<'de, D, T, K, V, I>(deserializer: D) -> Result<T, D::Error>
where D: Deserializer<'de>, T: DuplicateAggregates<K, V, I>, K: Deserialize<'de>, V: Deserialize<'de> + Default + IntoIterator<Item = I> + Extend<I>, I: Deserialize<'de> {
    struct MapVisitor<T, K, V> {
        marker: PhantomData<T>,
        map_key_type: PhantomData<K>,
        map_value_type: PhantomData<V>,
    }

    impl<'de, T, K, V, I> Visitor<'de> for MapVisitor<T, K, V>
    where T: DuplicateAggregates<K, V, I>, K: Deserialize<'de>, V: Deserialize<'de> + Default + IntoIterator<Item = I> + Extend<I>, I: Deserialize<'de> {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a map")
        }

        #[inline]
        fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
        where A: MapAccess<'de> {
            let mut values = Self::Value::new(access.size_hint());

            while let Some((key, value)) = access.next_entry()? {
                values.insert(key, value);
            }

            Ok(values)
        }
    }

    let visitor = MapVisitor {
        marker: PhantomData,
        map_key_type: PhantomData,
        map_value_type: PhantomData,
    };

    deserializer.deserialize_map(visitor)
}

/// Serialize the map with the default serializer
pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where T: Serialize, S: Serializer {
    value.serialize(serializer)
}