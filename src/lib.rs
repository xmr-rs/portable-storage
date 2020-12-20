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

use bytes::{Buf, BufMut, BytesMut};
use linked_hash_map::LinkedHashMap;
use std::{convert::TryFrom, ops::Index};
use thiserror::Error;

pub mod de;
pub mod ser;

pub use de::from_section;
pub use ser::to_section;

#[macro_export]
macro_rules! ensure_eof {
    ($buf:expr, $needed:expr) => {
        if $buf.remaining() < $needed {
            return Err($crate::Error::UnexpectedEof { needed: $needed });
        }
    };
}

pub mod header;
pub mod raw_size;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("reached EOF, needed {}", needed)]
    UnexpectedEof { needed: usize },
    #[error("the header isn't valid")]
    InvalidHeader,
    #[error("the storage entry serialize type isn't valid ({:X})", _0)]
    InvalidSerializeType(u8),
    #[error("the array serialize type isn't valid ({:X})", _0)]
    InvalidArrayType(u8),
    #[error("the storage entry size is too big for this machine ({})", _0)]
    StorageEntryTooBig(u64),
    #[error("wrong type sequence")]
    WrongTypeSequence,
}

const SERIALIZE_TYPE_INT64: u8 = 1;
const SERIALIZE_TYPE_INT32: u8 = 2;
const SERIALIZE_TYPE_INT16: u8 = 3;
const SERIALIZE_TYPE_INT8: u8 = 4;
const SERIALIZE_TYPE_UINT64: u8 = 5;
const SERIALIZE_TYPE_UINT32: u8 = 6;
const SERIALIZE_TYPE_UINT16: u8 = 7;
const SERIALIZE_TYPE_UINT8: u8 = 8;
const SERIALIZE_TYPE_DOUBLE: u8 = 9;
const SERIALIZE_TYPE_STRING: u8 = 10;
const SERIALIZE_TYPE_BOOL: u8 = 11;
const SERIALIZE_TYPE_OBJECT: u8 = 12;
const SERIALIZE_TYPE_ARRAY: u8 = 13;
const SERIALIZE_FLAG_ARRAY: u8 = 0x80;

#[derive(Debug, Clone)]
pub enum StorageEntry {
    U64(u64),
    U32(u32),
    U16(u16),
    U8(u8),
    I64(i64),
    I32(i32),
    I16(i16),
    I8(i8),
    Double(f64),
    Bool(bool),
    Buf(Vec<u8>),
    Array(Array),
    Section(Section),
}

impl StorageEntry {
    fn read<B: Buf>(buf: &mut B) -> Result<StorageEntry> {
        ensure_eof!(buf, 1);
        let serialize_type = buf.get_u8();
        if serialize_type & SERIALIZE_FLAG_ARRAY == SERIALIZE_FLAG_ARRAY {
            let arr = Array::read::<B>(buf, serialize_type)?;
            return Ok(StorageEntry::Array(arr));
        }

        Self::read_entry_raw::<B>(buf, serialize_type)
    }

    fn read_entry_raw<B: Buf>(buf: &mut B, serialize_type: u8) -> Result<StorageEntry> {
        let entry = match serialize_type {
            SERIALIZE_TYPE_INT64 => {
                ensure_eof!(buf, 8);
                StorageEntry::I64(buf.get_i64_le())
            }
            SERIALIZE_TYPE_INT32 => {
                ensure_eof!(buf, 4);
                StorageEntry::I32(buf.get_i32_le())
            }
            SERIALIZE_TYPE_INT16 => {
                ensure_eof!(buf, 2);
                StorageEntry::I16(buf.get_i16_le())
            }
            SERIALIZE_TYPE_INT8 => {
                ensure_eof!(buf, 1);
                StorageEntry::I8(buf.get_i8())
            }
            SERIALIZE_TYPE_UINT64 => {
                ensure_eof!(buf, 8);
                StorageEntry::U64(buf.get_u64_le())
            }
            SERIALIZE_TYPE_UINT32 => {
                ensure_eof!(buf, 4);
                StorageEntry::U32(buf.get_u32_le())
            }
            SERIALIZE_TYPE_UINT16 => {
                ensure_eof!(buf, 2);
                StorageEntry::U16(buf.get_u16_le())
            }
            SERIALIZE_TYPE_UINT8 => {
                ensure_eof!(buf, 1);
                StorageEntry::U8(buf.get_u8())
            }
            SERIALIZE_TYPE_DOUBLE => {
                ensure_eof!(buf, 8);
                StorageEntry::Double(buf.get_f64_le())
            }
            SERIALIZE_TYPE_STRING => {
                let b = read_buf::<B>(buf)?;
                StorageEntry::Buf(b)
            }
            SERIALIZE_TYPE_BOOL => {
                ensure_eof!(buf, 1);
                StorageEntry::Bool(buf.get_u8() != 0)
            }
            SERIALIZE_TYPE_OBJECT => StorageEntry::Section(Section::read::<B>(buf)?),
            SERIALIZE_TYPE_ARRAY => {
                ensure_eof!(buf, 1);

                let serialize_type = buf.get_u8();
                if serialize_type & SERIALIZE_FLAG_ARRAY != SERIALIZE_FLAG_ARRAY {
                    return Err(Error::WrongTypeSequence);
                }

                let arr = Array::read::<B>(buf, serialize_type)?;
                StorageEntry::Array(arr)
            }
            _ => {
                return Err(Error::InvalidSerializeType(serialize_type));
            }
        };

        Ok(entry)
    }

    fn write(buf: &mut BytesMut, entry: &Self) {
        match entry {
            StorageEntry::U64(v) => {
                buf.reserve(9);
                buf.put_u8(SERIALIZE_TYPE_UINT64);
                buf.put_u64_le(*v);
            }
            StorageEntry::U32(v) => {
                buf.reserve(5);
                buf.put_u8(SERIALIZE_TYPE_UINT32);
                buf.put_u32_le(*v);
            }
            StorageEntry::U16(v) => {
                buf.reserve(3);
                buf.put_u8(SERIALIZE_TYPE_UINT16);
                buf.put_u16_le(*v);
            }
            StorageEntry::U8(v) => {
                buf.reserve(2);
                buf.put_u8(SERIALIZE_TYPE_UINT8);
                buf.put_u8(*v);
            }
            StorageEntry::I64(v) => {
                buf.reserve(9);
                buf.put_u8(SERIALIZE_TYPE_INT64);
                buf.put_i64_le(*v);
            }
            StorageEntry::I32(v) => {
                buf.reserve(5);
                buf.put_u8(SERIALIZE_TYPE_INT32);
                buf.put_i32_le(*v);
            }
            StorageEntry::I16(v) => {
                buf.reserve(3);
                buf.put_u8(SERIALIZE_TYPE_INT16);
                buf.put_i16_le(*v);
            }
            StorageEntry::I8(v) => {
                buf.reserve(2);
                buf.put_u8(SERIALIZE_TYPE_INT8);
                buf.put_i8(*v);
            }
            StorageEntry::Double(v) => {
                buf.reserve(9);
                buf.put_u8(SERIALIZE_TYPE_DOUBLE);
                buf.put_f64_le(*v);
            }
            StorageEntry::Bool(v) => {
                buf.reserve(2);
                buf.put_u8(SERIALIZE_TYPE_BOOL);
                buf.put_u8(if !v { 0 } else { 1 });
            }
            StorageEntry::Buf(v) => {
                buf.reserve(1);
                buf.put_u8(SERIALIZE_TYPE_STRING);
                write_buf(buf, v);
            }
            StorageEntry::Array(v) => {
                buf.reserve(1);
                buf.put_u8(SERIALIZE_TYPE_ARRAY);
                Array::write(buf, v);
            }
            StorageEntry::Section(v) => {
                buf.reserve(1);
                buf.put_u8(SERIALIZE_TYPE_OBJECT);
                Section::write(buf, v);
            }
        }
    }

    fn serialize_type(&self) -> u8 {
        match self {
            StorageEntry::U64(_) => SERIALIZE_TYPE_UINT64,
            StorageEntry::U32(_) => SERIALIZE_TYPE_UINT32,
            StorageEntry::U16(_) => SERIALIZE_TYPE_UINT16,
            StorageEntry::U8(_) => SERIALIZE_TYPE_UINT8,
            StorageEntry::I64(_) => SERIALIZE_TYPE_INT64,
            StorageEntry::I32(_) => SERIALIZE_TYPE_INT32,
            StorageEntry::I16(_) => SERIALIZE_TYPE_INT16,
            StorageEntry::I8(_) => SERIALIZE_TYPE_INT8,
            StorageEntry::Double(_) => SERIALIZE_TYPE_DOUBLE,
            StorageEntry::Bool(_) => SERIALIZE_TYPE_BOOL,
            StorageEntry::Buf(_) => SERIALIZE_TYPE_STRING,
            StorageEntry::Array(_) => SERIALIZE_TYPE_ARRAY,
            StorageEntry::Section(_) => SERIALIZE_TYPE_OBJECT,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Array {
    array: Vec<StorageEntry>,
    serialize_type: Option<u8>,
}

impl Array {
    pub fn new() -> Array {
        Default::default()
    }

    pub fn with_capacity(capacity: usize) -> Array {
        Array {
            array: Vec::with_capacity(capacity),
            serialize_type: None,
        }
    }

    pub fn len(&self) -> usize {
        self.array.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, entry: StorageEntry) -> std::result::Result<(), ()> {
        if let Some(serialize_type) = self.serialize_type {
            if serialize_type & SERIALIZE_FLAG_ARRAY != entry.serialize_type() {
                return Err(());
            }
        } else {
            self.serialize_type = Some(entry.serialize_type() | SERIALIZE_FLAG_ARRAY);
        }

        self.array.push(entry);
        Ok(())
    }

    fn read<B: Buf>(buf: &mut B, mut serialize_type: u8) -> Result<Array> {
        let orig_serialize_type = serialize_type;
        if serialize_type & SERIALIZE_FLAG_ARRAY != SERIALIZE_FLAG_ARRAY {
            return Err(Error::InvalidArrayType(serialize_type));
        } else {
            serialize_type &= !SERIALIZE_FLAG_ARRAY;
        }

        let size = raw_size::read::<B>(buf)
            .and_then(|size| usize::try_from(size).map_err(|_| Error::StorageEntryTooBig(size)))?;

        let mut array = Array {
            array: Vec::new(),
            serialize_type: Some(orig_serialize_type),
        };
        // TODO(jeandudey): same bug as in Section::read, check it out before
        // uncommenting this, potential DDoS.
        // array.array.reserve(size);

        for _ in 0..size {
            array
                .array
                .push(StorageEntry::read_entry_raw::<B>(buf, serialize_type)?);
        }

        Ok(array)
    }

    fn write(buf: &mut BytesMut, array: &Array) {
        buf.reserve(1);
        buf.put_u8(array.serialize_type.unwrap());
        raw_size::write(buf, array.array.len() as u64);
        for entry in array.array.iter() {
            StorageEntry::write(buf, &entry);
        }
    }
}

impl IntoIterator for Array {
    type Item = StorageEntry;

    type IntoIter = std::vec::IntoIter<StorageEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.array.into_iter()
    }
}

impl Index<usize> for Array {
    type Output = StorageEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.array[index]
    }
}

#[derive(Debug, Clone, Default)]
pub struct Section {
    pub entries: LinkedHashMap<String, StorageEntry>,
}

impl Section {
    pub fn new() -> Section {
        Default::default()
    }

    pub fn with_capacity(capacity: usize) -> Section {
        Section {
            entries: LinkedHashMap::with_capacity(capacity),
        }
    }

    /// Insernt an storage entry.
    pub fn insert<T: Into<StorageEntry>>(&mut self, name: String, entry: T) {
        self.entries.insert(name, entry.into());
    }

    /// Length of this section.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn read<B: Buf>(buf: &mut B) -> Result<Section> {
        let mut section = Section::new();
        let count = raw_size::read::<B>(buf).and_then(|count| {
            usize::try_from(count).map_err(|_| Error::StorageEntryTooBig(count))
        })?;

        // TODO(jeandudey): this statement gives some performance, but it's
        // disabled since it can be easily abused because we don't have a way
        // to check for the byte size of the sections count to check for EOF
        // and validity.
        //
        // Gentle reminder: check if Monero suffers from this same problem to
        // avoid a DDoS by triggering OOM errors.

        // section.entries.reserve(count);

        for _ in 0..count {
            let name = read_name::<B>(buf)?;
            let entry = StorageEntry::read::<B>(buf)?;
            section.entries.insert(name.clone(), entry);
        }

        Ok(section)
    }

    fn write(buf: &mut BytesMut, section: &Self) {
        raw_size::write(buf, section.entries.len() as u64);

        for (name, entry) in section.entries.iter() {
            write_name(buf, &*name);
            StorageEntry::write(buf, &entry);
        }
    }
}

impl IntoIterator for Section {
    type Item = (String, StorageEntry);

    type IntoIter = linked_hash_map::IntoIter<String, StorageEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl Index<&'static str> for Section {
    type Output = StorageEntry;

    fn index(&self, index: &'static str) -> &Self::Output {
        &self.entries[index]
    }
}

pub fn read<B: Buf>(buf: &mut B) -> Result<Section> {
    header::StorageBlockHeader::read::<B>(buf)?;
    Section::read::<B>(buf)
}

pub fn write(buf: &mut BytesMut, section: &Section) {
    header::StorageBlockHeader::write(buf);
    Section::write(buf, section);
}

fn read_name<B: Buf>(buf: &mut B) -> Result<String> {
    ensure_eof!(buf, 1);
    let length = buf.get_u8() as usize;
    ensure_eof!(buf, length);

    let s = String::from_utf8_lossy(&buf.bytes()[..length]).into_owned();
    buf.advance(length);
    Ok(s)
}

fn read_buf<B: Buf>(buf: &mut B) -> Result<Vec<u8>> {
    let length = raw_size::read::<B>(buf).and_then(|length| {
        usize::try_from(length).map_err(|_| Error::StorageEntryTooBig(length))
    })?;
    ensure_eof!(buf, length);

    let mut b = Vec::with_capacity(length);
    b.extend_from_slice(&buf.bytes()[..length]);
    buf.advance(length);
    Ok(b)
}

fn write_buf(buf: &mut BytesMut, b: &[u8]) {
    raw_size::write(buf, b.len() as u64);

    buf.reserve(b.len());
    buf.put(b);
}

fn write_name(buf: &mut BytesMut, name: &str) {
    buf.reserve(name.as_bytes().len() + 1);
    buf.put_u8(name.as_bytes().len() as u8);
    buf.put(name.as_bytes());
}
