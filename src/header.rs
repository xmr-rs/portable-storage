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

pub const PORTABLE_STORAGE_SIGNATUREA: u32 = 0x0101_1101;
pub const PORTABLE_STORAGE_SIGNATUREB: u32 = 0x0102_0101;
pub const PORTABLE_STORAGE_FORMAT_VER: u8 = 1;
pub const PORTABLE_STORAGE_BLOCK_HEADER_LENGTH: usize = 4 + 4 + 1;

#[derive(Debug)]
pub struct StorageBlockHeader {
    pub signature_a: u32,
    pub signature_b: u32,
    pub version: u8,
}

impl StorageBlockHeader {
    pub fn is_valid_signature_a(&self) -> bool {
        self.signature_a == PORTABLE_STORAGE_SIGNATUREA
    }

    pub fn is_valid_signature_b(&self) -> bool {
        self.signature_a == PORTABLE_STORAGE_SIGNATUREB
    }

    pub fn is_valid_version(&self) -> bool {
        self.version == PORTABLE_STORAGE_FORMAT_VER
    }

    pub fn read<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        ensure_eof!(buf, PORTABLE_STORAGE_BLOCK_HEADER_LENGTH);

        let hdr = StorageBlockHeader {
            signature_a: buf.get_u32_le(),
            signature_b: buf.get_u32_le(),
            version: buf.get_u8(),
        };

        if (hdr.is_valid_signature_a() || hdr.is_valid_signature_b()) && hdr.is_valid_version() {
            Ok(hdr)
        } else {
            Err(Error::InvalidHeader)
        }
    }

    pub fn write(buf: &mut BytesMut) {
        buf.reserve(PORTABLE_STORAGE_BLOCK_HEADER_LENGTH);
        buf.put_u32_le(PORTABLE_STORAGE_SIGNATUREA);
        buf.put_u32_le(PORTABLE_STORAGE_SIGNATUREB);
        buf.put_u8(PORTABLE_STORAGE_FORMAT_VER);
    }
}
