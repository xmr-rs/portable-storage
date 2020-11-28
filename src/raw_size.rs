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

//! # Raw size
//!
//! "Raw size" (that's how the monero's epee library calls it) are a form of variable size integers
//! (VarInt) used in the portable storage serialization format.
//!
//! # Usage
//!
//! This API is used internally by the serializer but can also be used directly if needed, the
//! usage is very simple, to write a raw size into a `bytes::BytesMut` buffer you simply do:
//!
//! ```rust
//! use bytes::BytesMut;
//!
//! let mut buf = BytesMut::new();
//! let our_int_value = 10;
//! portable_storage::raw_size::write(&mut buf, our_int_value);
//! ```
//!
//! And to read a raw size from a `bytes::Buf`:
//!
//! ```rust
//! use bytes::Bytes;
//!
//! const BUF: &[u8] = &[0xFC];
//!
//! let mut bytes = Bytes::from(BUF);
//! let value = portable_storage::raw_size::read(&mut bytes).unwrap();
//! assert_eq!(value, 63);
//! ```

use crate::Error;
use bytes::{Buf, BufMut, BytesMut};

/// The size in bits of the raw size marker.
pub const MARK_BIT_SIZE: usize = 2;
/// The mask of the marker.
pub const MARK_MASK: u8 = 0x03;
/// Mark type for a single byte raw size.
pub const MARK_U8: u8 = 0;
/// Mark type for a two bytes raw size.
pub const MARK_U16: u8 = 1;
/// Mark type for a four bytes raw size.
pub const MARK_U32: u8 = 2;
/// Mark type for a eight bytes raw size.
pub const MARK_U64: u8 = 3;

/// Maximum integer value that can be stored on a single byte.
pub const U8_MAX: u64 = (1 << (8 - MARK_BIT_SIZE)) - 1;
/// Maximum integer value that can be stored on two bytes.
pub const U16_MAX: u64 = (1 << (16 - MARK_BIT_SIZE)) - 1;
/// Maximum integer value that can be stored on four bytes.
pub const U32_MAX: u64 = (1 << (32 - MARK_BIT_SIZE)) - 1;
/// Maximum integer value that can be stored on eight bytes.
pub const U64_MAX: u64 = (1 << (64 - MARK_BIT_SIZE)) - 1;

/// Reads a "raw size" value from `buf`.
///
/// # Errors
///
/// This function may return an `Error::UnexpectedEof` error if `buf` doesn't
/// hold the needed bytes that the raw size marker specifies.
pub fn read<B: Buf>(buf: &mut B) -> Result<u64, Error> {
    ensure_eof!(buf, 1);
    let mark = buf.bytes()[0] & MARK_MASK;

    match mark {
        MARK_U8 => Ok((buf.get_u8() >> 2) as u64),
        MARK_U16 => {
            ensure_eof!(buf, 2);
            Ok((buf.get_u16_le() >> 2) as u64)
        }
        MARK_U32 => {
            ensure_eof!(buf, 4);
            Ok((buf.get_u32_le() >> 2) as u64)
        }
        MARK_U64 => {
            ensure_eof!(buf, 8);
            Ok(buf.get_u64_le() >> 2)
        }
        // The MARK_MASK ensures no other values are valid,
        // so it's unreachable.
        _ => unreachable!(),
    }
}

/// Writes the value onto `buf` as a "raw size" integer.
///
/// # Panics
///
/// This function will panic if the provided `val` value is higher or equal to
/// `U64_MAX` which is the maximum value that can be stored.
pub fn write(buf: &mut BytesMut, val: u64) {
    if val <= U8_MAX {
        buf.reserve(1);
        buf.put_u8(((val as u8) << 2) | MARK_U8);
    } else if val <= U16_MAX {
        buf.reserve(2);
        buf.put_u16_le(((val as u16) << 2) | MARK_U16 as u16);
    } else if val <= U32_MAX {
        buf.reserve(4);
        buf.put_u32_le(((val as u32) << 2) | MARK_U32 as u32);
    } else if val <= U64_MAX {
        buf.reserve(8);
        buf.put_u64_le(((val as u64) << 2) | MARK_U64 as u64);
    } else {
        panic!("the value is too big to be stored on a raw size variable integer");
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        const SIZES: &[(u64, usize)] = &[(U8_MAX, 1), (U16_MAX, 2), (U32_MAX, 4), (U64_MAX, 8)];

        for (value, size_in_bytes) in SIZES {
            let mut buf = BytesMut::new();
            write(&mut buf, *value);
            assert_eq!(buf.len(), *size_in_bytes);

            let mut buf = buf.freeze();
            let readed_value = read(&mut buf).unwrap();
            assert_eq!(readed_value, *value);
        }
    }

    #[test]
    #[should_panic]
    fn too_big() {
        let mut buf = BytesMut::new();
        write(&mut buf, U64_MAX + 1);
    }
}
