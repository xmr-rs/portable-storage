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

use serde::{
    de::{Deserialize, Deserializer, Error, Visitor},
    ser::{Serialize, Serializer},
};
use std::fmt;

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct BytesUuid(pub uuid::Uuid);

impl From<uuid::Uuid> for BytesUuid {
    fn from(v: uuid::Uuid) -> BytesUuid {
        BytesUuid(v)
    }
}

impl<'de> Deserialize<'de> for BytesUuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UuidVisitor;

        impl<'de> Visitor<'de> for UuidVisitor {
            type Value = BytesUuid;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "an uuid")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                uuid::Uuid::from_slice(v)
                    .map(BytesUuid::from)
                    .map_err(E::custom)
            }
        }

        deserializer.deserialize_bytes(UuidVisitor)
    }
}

impl Serialize for BytesUuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.0.as_bytes())
    }
}
