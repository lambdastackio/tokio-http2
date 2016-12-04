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

use std::{slice, mem, fmt};

use byteorder::ByteOrder;

use http2::kind::*;
use http2::flag::*;
use http2::frame::*;

use http2::Error;
use http2::ErrorCode;
use http2::SizeIncrement;
use http2::StreamIdentifier;
use http2::ParserSettings;
use http2::encode_u64;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Payload<'a> {
    Data {
        data: &'a [u8]
    },
    Headers {
        priority: Option<Priority>,
        block: &'a [u8]
    },
    Priority(Priority),
    Reset(ErrorCode),
    Settings(&'a [Setting]),
    PushPromise {
        promised: StreamIdentifier,
        block: &'a [u8]
    },
    Ping(u64),
    GoAway {
        last: StreamIdentifier,
        error: ErrorCode,
        data: &'a [u8]
    },
    WindowUpdate(SizeIncrement),
    Continuation(&'a [u8]),
    Unregistered(&'a [u8])
}

const PRIORITY_BYTES: u32 = 5;
const PADDING_BYTES: u32 = 1;

impl<'a> Payload<'a> {
    #[inline]
    pub fn kind(&self) -> Kind {
        use self::Payload::*;

        match *self {
            Data { .. } => Kind::Data,
            Headers { .. } => Kind::Headers,
            Priority(..) => Kind::Priority,
            Reset(..) => Kind::Reset,
            Settings(..) => Kind::Settings,
            PushPromise { .. } => Kind::PushPromise,
            Ping(..) => Kind::Ping,
            GoAway { .. } => Kind::GoAway,
            WindowUpdate(_) => Kind::WindowUpdate,
            Continuation(_) => Kind::Continuation,
            Unregistered(_) => Kind::Unregistered
        }
    }

    #[inline]
    pub fn parse(header: FrameHeader, mut buf: &'a [u8]) -> Result<Payload<'a>, Error> {
        let settings = ParserSettings {
            padding: header.flag.contains(Flag::padded()),
            priority: header.flag.contains(Flag::priority())
        };

        if buf.len() < header.length as usize {
            return Err(Error::Short)
        }

        let min_payload_length =
            if settings.priority && settings.padding {
                PRIORITY_BYTES + PADDING_BYTES
            } else if settings.priority {
                PRIORITY_BYTES
            } else if settings.padding {
                PADDING_BYTES
            } else {
                0
            };

        if header.length < min_payload_length {
            return Err(Error::PayloadLengthTooShort)
        }

        buf = &buf[..header.length as usize];

        match header.kind {
            Kind::Data => Payload::parse_data(header, buf, settings),
            Kind::Headers => Payload::parse_headers(header, buf, settings),
            Kind::Priority => {
                let (_, priority) = try!(Priority::parse(true, buf));
                Ok(Payload::Priority(priority.unwrap()))
            },
            Kind::Reset => Payload::parse_reset(header, buf),
            Kind::Settings => Payload::parse_settings(header, buf),
            Kind::Ping => Payload::parse_ping(header, buf),
            Kind::GoAway => Payload::parse_goaway(header, buf),
            Kind::WindowUpdate => Payload::parse_window_update(header, buf),
            Kind::PushPromise => Payload::parse_push_promise(header, buf, settings),
            Kind::Continuation => Ok(Payload::Continuation(buf)),
            Kind::Unregistered => Ok(Payload::Unregistered(buf))
        }
    }

    #[inline]
    pub fn encode(&self, buf: &mut [u8]) -> usize {
        match *self {
            Payload::Data { ref data } => { encode_memory(data, buf) },
            Payload::Headers { ref priority, ref block } => {
                let priority_wrote = priority.map(|p| { p.encode(buf) }).unwrap_or(0);
                let block_wrote = encode_memory(block, &mut buf[priority_wrote..]);
                priority_wrote + block_wrote
            },
            Payload::Reset(ref err) => { err.encode(buf) },
            Payload::Settings(ref settings) => {
                encode_memory(Setting::to_bytes(settings), buf)
            },
            Payload::Ping(data) => { encode_u64(buf, data) },
            Payload::GoAway { ref data, ref last, ref error } => {
                let last_wrote = last.encode(buf);
                let buf = &mut buf[last_wrote..];

                let error_wrote = error.encode(buf);
                let buf = &mut buf[error_wrote..];

                encode_memory(data, buf) + last_wrote + error_wrote
            },
            Payload::WindowUpdate(ref increment) => { increment.encode(buf) },
            Payload::PushPromise { ref promised, ref block } => {
                promised.encode(buf);
                encode_memory(block, &mut buf[4..]) + 4
            },
            Payload::Priority(ref priority) => { priority.encode(buf) },
            Payload::Continuation(ref block) => { encode_memory(block, buf) },
            Payload::Unregistered(ref block) => { encode_memory(block, buf) }
        }
    }

    #[inline]
    /// How many bytes this Payload would be encoded.
    pub fn encoded_len(&self) -> usize {
        use self::Payload::*;

        match *self {
            Data { ref data } => { data.len() },
            Headers { ref priority, ref block } => {
                let priority_len = if priority.is_some() { 5 } else { 0 };
                priority_len + block.len()
            },
            Reset(_) => 4,
            Settings(ref settings) => settings.len() * mem::size_of::<Setting>(),
            Ping(_) => 8,
            GoAway { ref data, .. } => 4 + 4 + data.len(),
            WindowUpdate(_) => 4,
            PushPromise { ref block, .. } => 4 + block.len(),
            Priority(_) => 5,
            Continuation(ref block) => block.len(),
            Unregistered(ref block) => block.len()
        }
    }

    #[inline]
    pub fn padded(&self) -> Option<u32> {
        None
    }

    #[inline]
    pub fn priority(&self) -> Option<&Priority> {
        match *self {
            Payload::Priority(ref priority) => Some(priority),
            Payload::Headers { ref priority, .. } => priority.as_ref(),
            _ => None
        }
    }

    #[inline]
    fn parse_data(header: FrameHeader, buf: &'a [u8],
                  settings: ParserSettings) -> Result<Payload<'a>, Error> {
        Ok(Payload::Data {
            data: try!(trim_padding(settings, header, buf))
        })
    }

    #[inline]
    fn parse_headers(header: FrameHeader, mut buf: &'a [u8],
                     settings: ParserSettings) -> Result<Payload<'a>, Error> {
        buf = try!(trim_padding(settings, header, buf));
        let (buf, priority) = try!(Priority::parse(settings.priority, buf));
        Ok(Payload::Headers {
            priority: priority,
            block: buf
        })
    }

    #[inline]
    fn parse_reset(header: FrameHeader,
                   buf: &'a [u8]) -> Result<Payload<'a>, Error> {
        if header.length < 4 {
            return Err(Error::PayloadLengthTooShort)
        }

        Ok(Payload::Reset(ErrorCode::parse(buf)))
    }

    #[inline]
    fn parse_settings(header: FrameHeader,
                      buf: &'a [u8]) -> Result<Payload<'a>, Error> {
        if header.length % mem::size_of::<Setting>() as u32 != 0 {
            return Err(Error::PartialSettingLength)
        }

        Ok(Payload::Settings(Setting::from_bytes(&buf[..header.length as usize])))
    }

    #[inline]
    fn parse_ping(header: FrameHeader,
                  buf: &'a [u8]) -> Result<Payload<'a>, Error> {
        if header.length != 8 {
            return Err(Error::InvalidPayloadLength)
        }

        let data = ::byteorder::BigEndian::read_u64(buf);
        Ok(Payload::Ping(data))
    }

    #[inline]
    fn parse_goaway(header: FrameHeader,
                    buf: &'a [u8]) -> Result<Payload<'a>, Error> {
        if header.length < 8 {
            return Err(Error::PayloadLengthTooShort)
        }

        let last = StreamIdentifier::parse(buf);
        let error = ErrorCode::parse(&buf[4..]);
        let rest = &buf[8..];

        Ok(Payload::GoAway {
            last: last,
            error: error,
            data: rest
        })
    }

    #[inline]
    fn parse_window_update(header: FrameHeader,
                           buf: &'a [u8]) -> Result<Payload<'a>, Error> {
        if header.length != 4 {
            return Err(Error::InvalidPayloadLength)
        }

        Ok(Payload::WindowUpdate(SizeIncrement::parse(buf)))
    }

    #[inline]
    fn parse_push_promise(header: FrameHeader, mut buf: &'a [u8],
                          settings: ParserSettings) -> Result<Payload<'a>, Error> {
        buf = try!(trim_padding(settings, header, buf));

        if buf.len() < 4 {
            return Err(Error::PayloadLengthTooShort)
        }

        let promised = StreamIdentifier::parse(buf);
        let block = &buf[4..];

        Ok(Payload::PushPromise {
             promised: promised,
             block: block
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Priority {
    exclusive: bool,
    dependency: StreamIdentifier,
    weight: u8
}

impl Priority {
    #[inline]
    pub fn parse(present: bool, buf: &[u8]) -> Result<(&[u8], Option<Priority>), Error> {
        if present {
            Ok((&buf[5..], Some(Priority {
                // Most significant bit.
                exclusive: buf[0] & 0x7F != buf[0],
                dependency: StreamIdentifier::parse(buf),
                weight: buf[4]
            })))
        } else {
            Ok((buf, None))
        }
    }

    #[inline]
    pub fn encode(&self, buf: &mut [u8]) -> usize {
        let mut dependency = self.dependency;
        if self.exclusive { dependency.0 |= 1 << 31 }

        dependency.encode(buf);
        buf[PRIORITY_BYTES as usize - 1] = self.weight;

        PRIORITY_BYTES as usize
    }
}

// Settings are (u16, u32) in memory.
#[repr(packed)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Setting {
    identifier: u16,
    value: u32
}

impl fmt::Debug for Setting {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.identifier(), f)
    }
}

impl Setting {
    #[inline]
    pub fn new(identifier: SettingIdentifier, value: u32) -> Setting {
        Setting {
            identifier: identifier as u16,
            value: value,
        }
    }

    #[inline]
    pub fn identifier(&self) -> Option<SettingIdentifier> {
        match self.identifier {
            0x1 => Some(SettingIdentifier::HeaderTableSize),
            0x2 => Some(SettingIdentifier::EnablePush),
            0x3 => Some(SettingIdentifier::MaxConcurrentStreams),
            0x4 => Some(SettingIdentifier::InitialWindowSize),
            0x5 => Some(SettingIdentifier::MaxFrameSize),
            _ => None
        }
    }

    #[inline]
    pub fn value(&self) -> u32 {
        self.value
    }

    #[inline]
    fn to_bytes(settings: &[Setting]) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                settings.as_ptr() as *const u8,
                settings.len() * mem::size_of::<Setting>())
        }
    }

    #[inline]
    fn from_bytes(bytes: &[u8]) -> &[Setting] {
        unsafe {
            slice::from_raw_parts(
                bytes.as_ptr() as *const Setting,
                bytes.len() / mem::size_of::<Setting>())
        }
    }
}

#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SettingIdentifier {
    HeaderTableSize = 0x1,
    EnablePush = 0x2,
    MaxConcurrentStreams = 0x3,
    InitialWindowSize = 0x4,
    MaxFrameSize = 0x5
}

/*
fn leak<T>(buf: Vec<T>) -> &'static [T] {
    let result = unsafe { mem::transmute::<&[T], &'static [T]>(&*buf) };
    mem::forget(buf);
    result
}
*/

#[inline]
fn trim_padding(settings: ParserSettings, header: FrameHeader,
                buf: &[u8]) -> Result<&[u8], Error> {
    if settings.padding {
        let pad_length = buf[0];
        if pad_length as u32 > header.length {
            Err(Error::TooMuchPadding(pad_length))
        } else {
            Ok(&buf[1..header.length as usize - pad_length as usize])
        }
    } else {
        Ok(buf)
    }
}

#[inline]
fn encode_memory(src: &[u8], mut dst: &mut [u8]) -> usize {
    use std::io::Write;
    dst.write(src).unwrap()
}
