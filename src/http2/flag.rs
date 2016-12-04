// Copyright 2016 LambdaStack All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! NB: This code is changing so please do not depend on it at this time!

bitflags! {
    pub flags Flag: u8 {
        const END_STREAM = 0x1,
        const ACK = 0x1,
        const END_HEADERS = 0x4,
        const PADDED = 0x8,
        const PRIORITY = 0x20
    }
}

impl Flag {
    pub fn new(data: u8) -> Result<Flag, ()> {
        match Flag::from_bits(data) {
            Some(v) => Ok(v),
            None => Err(())
        }
    }

    // Note that ACK and END_STREAM are the same value, but they are only present
    // on different frame types.
    pub fn ack() -> Flag { ACK }
    pub fn end_stream() -> Flag { END_STREAM }
    pub fn end_headers() -> Flag { END_HEADERS }
    pub fn padded() -> Flag { PADDED }
    pub fn priority() -> Flag { PRIORITY }
}
