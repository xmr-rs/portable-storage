// Copyright 2018-2020 Jean Pierre Dudey <me@jeandudey.tech>
// Copyright 2020 Artem Vorotnikov <artem@vorotnikov.me>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{Section, StorageEntry};
use linked_hash_map::LinkedHashMap;
use serde::{
    de::{
        value::Error, DeserializeSeed, Deserializer, Error as ErrorTrait, MapAccess, SeqAccess,
        Visitor,
    },
    forward_to_deserialize_any, Deserialize,
};

pub fn from_section<'de, T: Deserialize<'de>>(section: Section) -> Result<T, Error> {
    T::deserialize(SectionDeserializer(section))
}

macro_rules! unsupported {
    ($($method:ident)+) => {
        $(
        fn $method<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
            where V: Visitor<'de> {
            Err(Error::custom(concat!("`", stringify!($method), "` isn't supported")))
        }
        )+
    }
}

struct SectionDeserializer(Section);

impl<'de> Deserializer<'de> for SectionDeserializer {
    type Error = Error;

    unsupported! {
        deserialize_any deserialize_bool deserialize_i8 deserialize_i16
        deserialize_i32 deserialize_i64 deserialize_u8 deserialize_u16
        deserialize_u32 deserialize_u64 deserialize_f32 deserialize_f64
        deserialize_char deserialize_str deserialize_string deserialize_bytes
        deserialize_byte_buf deserialize_option deserialize_unit deserialize_seq
        deserialize_map deserialize_identifier deserialize_ignored_any
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::custom("`deserialize_unit_struct` isn't supported"))
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::custom(
            "`deserialize_newtype_struct` isn't supported",
        ))
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::custom("`deserialize_tuple` isn't supported"))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::custom("`deserialize_tuple_struct` isn't supported"))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let iter = self.0.into_iter();
        visitor.visit_map(MapDeserializer { iter, value: None })
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::custom("`deserialize_enum` isn't supported"))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

pub struct StorageEntryDeserializer(StorageEntry);

impl<'de> Deserializer<'de> for StorageEntryDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            StorageEntry::U64(v) => visitor.visit_u64(v),
            StorageEntry::U32(v) => visitor.visit_u32(v),
            StorageEntry::U16(v) => visitor.visit_u16(v),
            StorageEntry::U8(v) => visitor.visit_u8(v),
            StorageEntry::I64(v) => visitor.visit_i64(v),
            StorageEntry::I32(v) => visitor.visit_i32(v),
            StorageEntry::I16(v) => visitor.visit_i16(v),
            StorageEntry::I8(v) => visitor.visit_i8(v),
            StorageEntry::Double(v) => visitor.visit_f64(v),
            StorageEntry::Bool(v) => visitor.visit_bool(v),
            StorageEntry::Buf(v) => visitor.visit_byte_buf(v),
            StorageEntry::Array(v) => visitor.visit_seq(ArrayDeserializer(v.into_iter())),
            StorageEntry::Section(v) => visitor.visit_map(MapDeserializer {
                iter: v.into_iter(),
                value: None,
            }),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct ArrayDeserializer(<Vec<StorageEntry> as IntoIterator>::IntoIter);

impl<'de> SeqAccess<'de> for ArrayDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(element) = self.0.next() {
            seed.deserialize(StorageEntryDeserializer(element))
                .map(Some)
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.0.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct KeyDeserializer {
    key: String,
}

impl<'de> Deserializer<'de> for KeyDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(self.key.as_str())
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct MapDeserializer {
    iter: <LinkedHashMap<String, StorageEntry> as IntoIterator>::IntoIter,
    value: Option<StorageEntry>,
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.iter.next() {
            self.value = Some(value);
            let key_de = KeyDeserializer { key };
            seed.deserialize(key_de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .value
            .take()
            .ok_or_else(|| Error::custom("seed value is missing"))?;
        seed.deserialize(StorageEntryDeserializer(value))
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct TestVector0 {
        id: u8,
        transaction_proof: u64,
    }

    #[test]
    fn test_vector_0() {
        let mut section = Section::with_capacity(2);

        // TODO: change Section::insert signature
        // to use Into<String>.
        section.insert("id".to_owned(), StorageEntry::U8(56));
        section.insert("transaction_proof".to_owned(), StorageEntry::U64(1337));

        let test_vector_0: TestVector0 = from_section(section).unwrap();

        assert_eq!(test_vector_0.id, 56);
        assert_eq!(test_vector_0.transaction_proof, 1337);
    }
}
