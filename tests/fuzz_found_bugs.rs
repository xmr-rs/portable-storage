// Copyright 2020 Jean Pierre Dudey <me@jeandudey.tech>
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

use bytes::Bytes;

#[test]
fn fuzz_case_de_0() {
    const BUF: &[u8] = &[1, 1, 2, 1, 1, 122, 2, 1, 1, 1, 255, 2, 255, 255];
    let mut buf = Bytes::from(BUF);
    portable_storage::read(&mut buf).ok();
}

#[test]
fn fuzz_case_de_1() {
    const BUF: &[u8] = &[
        1, 1, 2, 1, 1, 50, 2, 1, 1, 50, 122, 2, 1, 1, 1, 255, 255, 255, 35, 255, 0, 1, 1, 142,
    ];
    let mut buf = Bytes::from(BUF);
    portable_storage::read(&mut buf).ok();
}

#[test]
fn fuzz_case_de_2() {
    const BUF: &[u8] = &[
        1, 1, 2, 1, 50, 1, 122, 2, 1, 1, 1, 2, 1, 1, 141, 1, 5, 1, 1, 91, 1, 50, 122,
    ];
    let mut buf = Bytes::from(BUF);
    portable_storage::read(&mut buf).ok();
}
