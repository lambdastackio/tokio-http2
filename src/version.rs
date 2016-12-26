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

//! HTTP Versions enum
//!
//! Instead of relying on typo-prone Strings, use expected HTTP versions as
//! the `HttpVersion` enum.
use std::fmt;

use self::HttpVersion::{Http09, Http10, Http11, H2, H2c};

/// Represents a version of the HTTP spec.
#[derive(PartialEq, PartialOrd, Copy, Clone, Eq, Ord, Hash, Debug)]
pub enum HttpVersion {
    /// `HTTP/0.9`
    Http09,
    /// `HTTP/1.0`
    Http10,
    /// `HTTP/1.1`
    Http11,
    /// `HTTP/2.0` over TLS
    H2,
    /// `HTTP/2.0` over cleartext
    H2c,
}

impl fmt::Display for HttpVersion {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(match *self {
            Http09 => "HTTP/0.9",
            Http10 => "HTTP/1.0",
            Http11 => "HTTP/1.1",
            H2 => "h2",
            H2c => "h2c",
        })
    }
}

impl Default for HttpVersion {
    fn default() -> HttpVersion {
        Http11
    }
}
