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

use http2::kind::*;
use http2::flag::*;
use http2::payload::*;

use http2::Error;
use http2::ErrorCode;
use http2::SizeIncrement;
use http2::StreamIdentifier;
use http2::FRAME_HEADER_BYTES;
use http2::encode_u24;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Frame<'a> {
    pub header: FrameHeader,
    pub payload: Payload<'a>
}

impl<'a> Frame<'a> {
    pub fn parse(header: FrameHeader, buf: &[u8]) -> Result<Frame, Error> {
        Ok(Frame {
            header: header,
            payload: try!(Payload::parse(header, buf))
        })
    }

    /// Encodes this Frame into a buffer.
    pub fn encode(&self, buf: &mut [u8]) -> usize {
        self.header.encode(buf);
        self.payload.encode(&mut buf[FRAME_HEADER_BYTES..]) + FRAME_HEADER_BYTES
    }

    /// How many bytes this Frame will use in a buffer when encoding.
    pub fn encoded_len(&self) -> usize {
        FRAME_HEADER_BYTES + self.payload.encoded_len()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FrameHeader {
    pub length: u32,
    pub kind: Kind,
    pub flag: Flag,
    pub id: StreamIdentifier,
}

impl FrameHeader {
    #[inline]
    pub fn parse(buf: &[u8]) -> Result<FrameHeader, Error> {
        if buf.len() < FRAME_HEADER_BYTES {
            return Err(Error::Short);
        }

        Ok(FrameHeader {
            length: ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | buf[2] as u32,
            kind: Kind::new(buf[3]),
            flag: try!(Flag::new(buf[4]).map_err(|()| { Error::BadFlag(buf[4]) })),
            id: StreamIdentifier::parse(&buf[5..])
        })
    }

    #[inline]
    pub fn encode(&self, buf: &mut [u8]) {
        encode_u24(buf, self.length);
        buf[3] = self.kind.encode();
        buf[4] = self.flag.bits();
        self.id.encode(&mut buf[5..]);
    }
}
