//! Exposes the struct `Decoder` that allows for HPACK-encoded header blocks to
//! be decoded into a header list.
//!
//! The decoder only follows HPACK rules, without performing any additional
//! (semantic) checks on the header name/value pairs, i.e. it considers the
//! headers as opaque octets.
//!
//! # Example
//!
//! A simple example of using the decoder that demonstrates its API:
//!
//! ```rust
//! use hpack::Decoder;
//! let mut decoder = Decoder::new();
//!
//! let header_list = decoder.decode(&[0x82, 0x84]).unwrap();
//!
//! assert_eq!(header_list, [
//!     (b":method".to_vec(), b"GET".to_vec()),
//!     (b":path".to_vec(), b"/".to_vec()),
//! ]);
//! ```
//!
//! A more complex example where the callback API is used, providing the client a
//! borrowed representation of each header, rather than an owned representation.
//!
//! ```rust
//! use hpack::Decoder;
//! let mut decoder = Decoder::new();
//!
//! let mut count = 0;
//! let header_list = decoder.decode_with_cb(&[0x82, 0x84], |name, value| {
//!     count += 1;
//!     match count {
//!         1 => {
//!             assert_eq!(&name[..], &b":method"[..]);
//!             assert_eq!(&value[..], &b"GET"[..]);
//!         },
//!         2 => {
//!             assert_eq!(&name[..], &b":path"[..]);
//!             assert_eq!(&value[..], &b"/"[..]);
//!         },
//!         _ => panic!("Did not expect more than two headers!"),
//!     };
//! });
//! ```

use std::num::Wrapping;
use std::borrow::Cow;

use super::huffman::HuffmanDecoder;
use super::huffman::HuffmanDecoderError;

use super::STATIC_TABLE;
use super::{StaticTable, HeaderTable};

/// Decodes an integer encoded with a given prefix size (in bits).
/// Assumes that the buffer `buf` contains the integer to be decoded,
/// with the first byte representing the octet that contains the
/// prefix.
///
/// Returns a tuple representing the decoded integer and the number
/// of bytes from the buffer that were used.
fn decode_integer(buf: &[u8], prefix_size: u8)
        -> Result<(usize, usize), DecoderError> {
    if prefix_size < 1 || prefix_size > 8 {
        return Err(
            DecoderError::IntegerDecodingError(
                IntegerDecodingError::InvalidPrefix));
    }
    if buf.len() < 1 {
        return Err(
            DecoderError::IntegerDecodingError(
                IntegerDecodingError::NotEnoughOctets));
    }

    // Make sure there's no overflow in the shift operation
    let Wrapping(mask) = if prefix_size == 8 {
        Wrapping(0xFF)
    } else {
        Wrapping(1u8 << prefix_size) - Wrapping(1)
    };
    let mut value = (buf[0] & mask) as usize;
    if value < (mask as usize) {
        // Value fits in the prefix bits.
        return Ok((value, 1));
    }

    // The value does not fit into the prefix bits, so we read as many following
    // bytes as necessary to decode the integer.
    // Already one byte used (the prefix)
    let mut total = 1;
    let mut m = 0;
    // The octet limit is chosen such that the maximum allowed *value* can
    // never overflow an unsigned 32-bit integer. The maximum value of any
    // integer that can be encoded with 5 octets is ~2^28
    let octet_limit = 5;

    for &b in buf[1..].iter() {
        total += 1;
        value += ((b & 127) as usize) * (1 << m);
        m += 7;

        if b & 128 != 128 {
            // Most significant bit is not set => no more continuation bytes
            return Ok((value, total));
        }

        if total == octet_limit {
            // The spec tells us that we MUST treat situations where the
            // encoded representation is too long (in octets) as an error.
            return Err(
                DecoderError::IntegerDecodingError(
                    IntegerDecodingError::TooManyOctets))
        }
    }

    // If we have reached here, it means the buffer has been exhausted without
    // hitting the termination condition.
    Err(DecoderError::IntegerDecodingError(
            IntegerDecodingError::NotEnoughOctets))
}

/// Decodes an octet string under HPACK rules of encoding found in the given
/// buffer `buf`.
///
/// It is assumed that the first byte in the buffer represents the start of the
/// encoded octet string.
///
/// Returns the decoded string in a newly allocated `Vec` and the number of
/// bytes consumed from the given buffer.
fn decode_string<'a>(buf: &'a [u8]) -> Result<(Cow<'a, [u8]>, usize), DecoderError> {
    let (len, consumed) = try!(decode_integer(buf, 7));
    // debug!("decode_string: Consumed = {}, len = {}", consumed, len);
    if consumed + len > buf.len() {
        return Err(
            DecoderError::StringDecodingError(
                StringDecodingError::NotEnoughOctets));
    }
    let raw_string = &buf[consumed..consumed + len];
    if buf[0] & 128 == 128 {
        // debug!("decode_string: Using the Huffman code");
        // Huffman coding used: pass the raw octets to the Huffman decoder
        // and return its result.
        let mut decoder = HuffmanDecoder::new();
        let decoded = match decoder.decode(raw_string) {
            Err(e) => {
                return Err(DecoderError::StringDecodingError(
                    StringDecodingError::HuffmanDecoderError(e)));
            },
            Ok(res) => res,
        };
        Ok((Cow::Owned(decoded), consumed + len))
    } else {
        // The octets were transmitted raw
        // debug!("decode_string: Raw octet string received");
        Ok((Cow::Borrowed(raw_string), consumed + len))
    }
}

/// Different variants of how a particular header field can be represented in
/// an HPACK encoding.
enum FieldRepresentation {
    Indexed,
    LiteralWithIncrementalIndexing,
    SizeUpdate,
    LiteralNeverIndexed,
    LiteralWithoutIndexing,
}

impl FieldRepresentation {
    /// Based on the given octet, returns the type of the field representation.
    ///
    /// The given octet should be the top-order byte of the header field that
    /// is about to be decoded.
    fn new(octet: u8) -> FieldRepresentation {
        if octet & 128 == 128 {
            // High-order bit set
            FieldRepresentation::Indexed
        } else if octet & 64 == 64 {
            // Bit pattern `01`
            FieldRepresentation::LiteralWithIncrementalIndexing
        } else if octet & 32 == 32 {
            // Bit pattern `001`
            FieldRepresentation::SizeUpdate
        } else if octet & 16 == 16 {
            // Bit pattern `0001`
            FieldRepresentation::LiteralNeverIndexed
        } else {
            // None of the top 4 bits is set => bit pattern `0000xxxx`
            FieldRepresentation::LiteralWithoutIndexing
        }
    }
}

/// Represents all errors that can be encountered while decoding an
/// integer.
#[derive(PartialEq)]
#[derive(Copy)]
#[derive(Clone)]
#[derive(Debug)]
pub enum IntegerDecodingError {
    /// 5.1. specifies that "excessively large integer decodings" MUST be
    /// considered an error (whether the size is the number of octets or
    /// value). This variant corresponds to the encoding containing too many
    /// octets.
    TooManyOctets,
    /// The variant corresponds to the case where the value of the integer
    /// being decoded exceeds a certain threshold.
    ValueTooLarge,
    /// When a buffer from which an integer was supposed to be encoded does
    /// not contain enough octets to complete the decoding.
    NotEnoughOctets,
    /// Only valid prefixes are [1, 8]
    InvalidPrefix,
}

/// Represents all errors that can be encountered while decoding an octet
/// string.
#[derive(PartialEq)]
#[derive(Copy)]
#[derive(Clone)]
#[derive(Debug)]
pub enum StringDecodingError {
    NotEnoughOctets,
    HuffmanDecoderError(HuffmanDecoderError),
}

/// Represents all errors that can be encountered while performing the decoding
/// of an HPACK header set.
#[derive(PartialEq)]
#[derive(Copy)]
#[derive(Clone)]
#[derive(Debug)]
pub enum DecoderError {
    HeaderIndexOutOfBounds,
    IntegerDecodingError(IntegerDecodingError),
    StringDecodingError(StringDecodingError),
    /// The size of the dynamic table can never be allowed to exceed the max
    /// size mandated to the decoder by the protocol. (by perfroming changes
    /// made by SizeUpdate blocks).
    InvalidMaxDynamicSize,
}

/// The result returned by the `decode` method of the `Decoder`.
pub type DecoderResult = Result<Vec<(Vec<u8>, Vec<u8>)>, DecoderError>;

/// Decodes headers encoded using HPACK.
///
/// For now, incremental decoding is not supported, i.e. it is necessary
/// to pass in the entire encoded representation of all headers to the
/// decoder, rather than processing it piece-by-piece.
pub struct Decoder<'a> {
    // The dynamic table will own its own copy of headers
    header_table: HeaderTable<'a>,
}

/// Represents a decoder of HPACK encoded headers. Maintains the state
/// necessary to correctly decode subsequent HPACK blocks.
impl<'a> Decoder<'a> {
    /// Creates a new `Decoder` with all settings set to default values.
    pub fn new() -> Decoder<'a> {
        Decoder::with_static_table(STATIC_TABLE)
    }

    /// Creates a new `Decoder` with the given slice serving as its static
    /// table.
    ///
    /// The slice should contain tuples where the tuple coordinates represent
    /// the header name and value, respectively.
    ///
    /// Note: in order for the final decoded content to match the encoding
    ///       (according to the standard, at least), this static table must be
    ///       the one defined in the HPACK spec.
    fn with_static_table(static_table: StaticTable<'a>) -> Decoder<'a> {
        Decoder {
            header_table: HeaderTable::with_static_table(static_table)
        }
    }

    /// Sets a new maximum dynamic table size for the decoder.
    pub fn set_max_table_size(&mut self, new_max_size: usize) {
        self.header_table.dynamic_table.set_max_table_size(new_max_size);
    }

    /// Decodes the headers found in the given buffer `buf`. Invokes the callback `cb` for each
    /// decoded header in turn, by providing it the header name and value as `Cow` byte array
    /// slices.
    ///
    /// The callback is free to decide how to handle the emitted header, however the `Cow` cannot
    /// outlive the closure body without assuming ownership or otherwise copying the contents.
    ///
    /// This is due to the fact that the header might be found (fully or partially) in the header
    /// table of the decoder, in which case the callback will have received a borrow of its
    /// contents. However, when one of the following headers is decoded, it is possible that the
    /// header table might have to be modified; so the borrow is only valid until the next header
    /// decoding begins, meaning until the end of the callback's body.
    ///
    /// If an error is encountered during the decoding of any header, decoding halts and the
    /// appropriate error is returned as the `Err` variant of the `Result`.
    pub fn decode_with_cb<F>(&mut self, buf: &[u8], mut cb: F) -> Result<(), DecoderError>
            where F: FnMut(Cow<[u8]>, Cow<[u8]>) {
        let mut current_octet_index = 0;

        while current_octet_index < buf.len() {
            // At this point we are always at the beginning of the next block
            // within the HPACK data.
            // The type of the block can always be determined from the first
            // byte.
            let initial_octet = buf[current_octet_index];
            let buffer_leftover = &buf[current_octet_index..];
            let consumed = match FieldRepresentation::new(initial_octet) {
                FieldRepresentation::Indexed => {
                    let ((name, value), consumed) =
                        try!(self.decode_indexed(buffer_leftover));
                    cb(Cow::Borrowed(name), Cow::Borrowed(value));

                    consumed
                },
                FieldRepresentation::LiteralWithIncrementalIndexing => {
                    let ((name, value), consumed) = {
                        let ((name, value), consumed) = try!(
                            self.decode_literal(buffer_leftover, true));
                        cb(Cow::Borrowed(&name), Cow::Borrowed(&value));

                        // Since we are to add the decoded header to the header table, we need to
                        // convert them into owned buffers that the decoder can keep internally.
                        let name = name.into_owned();
                        let value = value.into_owned();

                        ((name, value), consumed)
                    };
                    // This cannot be done in the same scope as the `decode_literal` call, since
                    // Rust cannot figure out that the `into_owned` calls effectively drop the
                    // borrow on `self` that the `decode_literal` return value had. Since adding
                    // a header to the table requires a `&mut self`, it fails to compile.
                    // Manually separating it out here works around it...
                    self.header_table.add_header(name, value);

                    consumed
                },
                FieldRepresentation::LiteralWithoutIndexing => {
                    let ((name, value), consumed) =
                        try!(self.decode_literal(buffer_leftover, false));
                    cb(name, value);

                    consumed
                },
                FieldRepresentation::LiteralNeverIndexed => {
                    // Same as the previous one, except if we were also a proxy
                    // we would need to make sure not to change the
                    // representation received here. We don't care about this
                    // for now.
                    let ((name, value), consumed) =
                        try!(self.decode_literal(buffer_leftover, false));
                    cb(name, value);

                    consumed
                },
                FieldRepresentation::SizeUpdate => {
                    // Handle the dynamic table size update...
                    self.update_max_dynamic_size(buffer_leftover)
                }
            };

            current_octet_index += consumed;
        }

        Ok(())
    }

    /// Decode the header block found in the given buffer.
    ///
    /// The decoded representation is returned as a sequence of headers, where both the name and
    /// value of each header is represented by an owned byte sequence (i.e. `Vec<u8>`).
    ///
    /// The buffer should represent the entire block that should be decoded.
    /// For example, in HTTP/2, all continuation frames need to be concatenated
    /// to a single buffer before passing them to the decoder.
    pub fn decode(&mut self, buf: &[u8]) -> DecoderResult {
        let mut header_list = Vec::new();

        try!(self.decode_with_cb(buf, |n, v| header_list.push((n.into_owned(), v.into_owned()))));

        Ok(header_list)
    }

    /// Decodes an indexed header representation.
    fn decode_indexed(&self, buf: &[u8])
            -> Result<((&[u8], &[u8]), usize), DecoderError> {
        let (index, consumed) = try!(decode_integer(buf, 7));
        // debug!("Decoding indexed: index = {}, consumed = {}", index, consumed);

        let (name, value) = try!(self.get_from_table(index));

        Ok(((name, value), consumed))
    }

    /// Gets the header (name, value) pair with the given index from the table.
    ///
    /// In this context, the "table" references the definition of the table
    /// where the static table is concatenated with the dynamic table and is
    /// 1-indexed.
    fn get_from_table(&self, index: usize)
            -> Result<(&[u8], &[u8]), DecoderError> {
        self.header_table.get_from_table(index).ok_or(
            DecoderError::HeaderIndexOutOfBounds)
    }

    /// Decodes a literal header representation from the given buffer.
    ///
    /// # Parameters
    ///
    /// - index: whether or not the decoded value should be indexed (i.e.
    ///   included in the dynamic table).
    fn decode_literal<'b>(&'b self, buf: &'b [u8], index: bool)
            -> Result<((Cow<[u8]>, Cow<[u8]>), usize), DecoderError> {
        let prefix = if index {
            6
        } else {
            4
        };
        let (table_index, mut consumed) = try!(decode_integer(buf, prefix));

        // First read the name appropriately
        let name = if table_index == 0 {
            // Read name string as literal
            let (name, name_len) = try!(decode_string(&buf[consumed..]));
            consumed += name_len;
            name
        } else {
            // Read name indexed from the table
            let (name, _) = try!(self.get_from_table(table_index));
            Cow::Borrowed(name)
        };

        // Now read the value as a literal...
        let (value, value_len) = try!(decode_string(&buf[consumed..]));
        consumed += value_len;

        Ok(((name, value), consumed))
    }

    /// Handles processing the `SizeUpdate` HPACK block: updates the maximum
    /// size of the underlying dynamic table, possibly causing a number of
    /// headers to be evicted from it.
    ///
    /// Assumes that the first byte in the given buffer `buf` is the first
    /// octet in the `SizeUpdate` block.
    ///
    /// Returns the number of octets consumed from the given buffer.
    fn update_max_dynamic_size(&mut self, buf: &[u8]) -> usize {
        let (new_size, consumed) = decode_integer(buf, 5).ok().unwrap();
        self.header_table.dynamic_table.set_max_table_size(new_size);

        // info!("Decoder changed max table size from {} to {}",
            //   self.header_table.dynamic_table.get_size(),
            //   new_size);

        consumed
    }
}
