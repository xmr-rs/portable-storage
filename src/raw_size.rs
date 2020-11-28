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

use crate::Error;
use bytes::{Buf, BufMut, BytesMut};

pub const PORTABLE_RAW_SIZE_MARK_MASK: u8 = 0x03;
pub const PORTABLE_RAW_SIZE_MARK_BYTE: u8 = 0;
pub const PORTABLE_RAW_SIZE_MARK_WORD: u8 = 1;
pub const PORTABLE_RAW_SIZE_MARK_DWORD: u8 = 2;
pub const PORTABLE_RAW_SIZE_MARK_INT64: u8 = 3;

pub fn read<B: Buf>(buf: &mut B) -> Result<usize, Error> {
    ensure_eof!(buf, 1);
    let mark = buf.bytes()[0] & PORTABLE_RAW_SIZE_MARK_MASK;

    match mark {
        PORTABLE_RAW_SIZE_MARK_BYTE => Ok((buf.get_u8() >> 2) as usize),
        PORTABLE_RAW_SIZE_MARK_WORD => {
            ensure_eof!(buf, 2);
            Ok((buf.get_u16_le() >> 2) as usize)
        }
        PORTABLE_RAW_SIZE_MARK_DWORD => {
            ensure_eof!(buf, 4);
            Ok((buf.get_u32_le() >> 2) as usize)
        }
        PORTABLE_RAW_SIZE_MARK_INT64 => {
            ensure_eof!(buf, 8);
            Ok((buf.get_u64_le() >> 2) as usize)
        }
        _ => unreachable!(),
    }
}

pub fn write(buf: &mut BytesMut, val: usize) {
    if val <= 63 {
        buf.reserve(1);
        buf.put_u8(((val as u8) << 2) | PORTABLE_RAW_SIZE_MARK_BYTE);
    } else if val <= 16383 {
        buf.reserve(2);
        buf.put_u16_le(((val as u16) << 2) | PORTABLE_RAW_SIZE_MARK_WORD as u16);
    } else if val <= 1_073_741_823 {
        buf.reserve(4);
        buf.put_u32_le(((val as u32) << 2) | PORTABLE_RAW_SIZE_MARK_DWORD as u32);
    } else if val as u64 <= 4_611_686_018_427_387_903 {
        buf.reserve(8);
        buf.put_u64_le(((val as u64) << 2) | PORTABLE_RAW_SIZE_MARK_INT64 as u64);
    } else {
        // XXX: Hope some day monero never uses a value too large.
        panic!("too large");
    }
}
