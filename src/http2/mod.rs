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

#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]

//! This library module provides HTTP/2 parsing and buffer frames used for HTTP/2.
//!
//! NB: This code is changing so please do not depend on it at this time!


pub const FRAME_HEADER_BYTES: usize = 9;

use byteorder::ByteOrder;
use byteorder;

pub mod kind;
pub mod flag;
pub mod payload;
pub mod frame;

use self::kind::*;
use self::flag::*;
use self::frame::*;
use self::payload::*;

/// Errors that can occur during parsing an HTTP/2 frame.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Error {
    /// A full frame header was not passed.
    Short,

    /// An unsupported value was set for the flag value.
    BadFlag(u8),

    /// An unsupported value was set for the frame kind.
    BadKind(u8),

    /// The padding length was larger than the frame-header-specified
    /// length of the payload.
    TooMuchPadding(u8),

    /// The payload length specified by the frame header was shorter than
    /// necessary for the parser settings specified and the frame type.
    ///
    /// This happens if, for instance, the priority flag is set and the
    /// header length is shorter than a stream dependency.
    ///
    /// `PayloadLengthTooShort` should be treated as a protocol error.
    PayloadLengthTooShort,

    /// The payload length specified by the frame header of a settings frame
    /// was not a round multiple of the size of a single setting.
    PartialSettingLength,

    /// The payload length specified by the frame header was not the
    /// value necessary for the specific frame type.
    InvalidPayloadLength
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParserSettings {
    padding: bool,
    priority: bool
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StreamIdentifier(pub u32);

impl StreamIdentifier {
    pub fn parse(buf: &[u8]) -> StreamIdentifier {
        StreamIdentifier(
            byteorder::BigEndian::read_u32(buf) & ((1 << 31) - 1)
        )
    }

    pub fn encode(&self, buf: &mut [u8]) -> usize {
        encode_u32(buf, self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ErrorCode(pub u32);

pub enum HttpError {
    Protocol,
    Internal,
    FlowControlError,
    SettingsTimeout,
}

impl ErrorCode {
    pub fn parse(buf: &[u8]) -> ErrorCode {
        ErrorCode(byteorder::BigEndian::read_u32(buf))
    }

    pub fn encode(&self, buf: &mut [u8]) -> usize {
        encode_u32(buf, self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SizeIncrement(pub u32);

impl SizeIncrement {
    pub fn parse(buf: &[u8]) -> SizeIncrement {
        SizeIncrement(byteorder::BigEndian::read_u32(buf))
    }

    pub fn encode(&self, buf: &mut [u8]) -> usize {
        encode_u32(buf, self.0)
    }
}

#[inline(always)]
pub fn encode_u24(buf: &mut [u8], val: u32) -> usize {
    buf[0] = (val >> 16) as u8;
    buf[1] = (val >> 8) as u8;
    buf[2] = val as u8;

    3
}

#[inline(always)]
pub fn encode_u32(buf: &mut [u8], val: u32) -> usize {
    byteorder::BigEndian::write_u32(buf, val);
    4
}

#[inline(always)]
pub fn encode_u64(buf: &mut [u8], val: u64) -> usize {
    byteorder::BigEndian::write_u64(buf, val);
    8
}
