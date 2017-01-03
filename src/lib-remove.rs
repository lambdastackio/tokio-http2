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

// NOTE: This attribute only needs to be set once.
#![doc(html_logo_url = "https://lambdastackio.github.io/static/images/lambdastack-200x200.png",
       html_favicon_url = "https://lambdastackio.github.io/static/images/favicon.ico",
       html_root_url = "https://lambdastackio.github.io/tokio-http2/tokio-http2/index.html")]

//! This library supplies the required modules to implement HTTP/2 which includes the HPACK header
//! compression that includes the Huffman encoding/decoding features.

//! # tokio_http2
//!
//! tokio_http2 is a fast, modern HTTP implementation written in and for Rust. It
//! is a low-level typesafe abstraction over raw HTTP, providing an elegant
//! layer over "stringly-typed" HTTP.
//!
//! tokio_http2 provides both a [Client](client/index.html) and a
//! [Server](server/index.html), along with a
//! [typed Headers system](header/index.html).
//!
//! If just getting started, consider looking over the **[Guide](../guide/)**.

extern crate cookie;
extern crate futures;
extern crate futures_cpupool;
extern crate httparse;
#[macro_use] extern crate language_tags;
#[macro_use] extern crate log;
#[macro_use] extern crate mime as mime_crate;
extern crate native_tls;
extern crate rustc_serialize as serialize;
extern crate time;
#[macro_use] extern crate tokio_core; // as tokio;
extern crate tokio_tls;
extern crate tokio_proto;
extern crate tokio_service;
extern crate unicase;
#[macro_use] extern crate url;
#[macro_use] extern crate bitflags;
extern crate byteorder;
extern crate net2;

// extern crate spmc;

// // #[cfg(feature = "serde-serialization")]
// // extern crate serde;
// // extern crate mio;
// // extern crate vecio;

// #[cfg(all(test, feature = "nightly"))]
// extern crate test;
// pub use client::Client;
// pub use http::Chunk;

pub use url::Url;
// pub use body::Body;
pub use error::{Result, Error};
// pub use header::Headers;
pub use method::Method::{self, Get, Head, Post, Delete};
pub use status::StatusCode::{self, Ok, BadRequest, NotFound};
// pub use server::Server;
pub use uri::RequestUri;
pub use version::HttpVersion;

// macro_rules! unimplemented {
//     () => ({
//         panic!("unimplemented")
//     });
//     ($msg:expr) => ({
//         unimplemented!("{}", $msg)
//     });
//     ($fmt:expr, $($arg:tt)*) => ({
//         panic!(concat!("unimplemented: ", $fmt), $($arg)*)
//     });
// }

// #[cfg(test)]
// mod mock;
// mod body;
// pub mod client;
// //pub mod net;

//pub mod http2;
//pub mod hpack;

pub type Body = Vec<u8>;
pub type ContentType = String;
pub type ContentLength = u64;
pub type Headers = Vec<(String, String)>;

pub mod error;
mod method;
pub mod header;
// mod http;
pub mod http;
pub mod status;
mod uri;
mod version;

/// Re-exporting the mime crate, for convenience.
pub mod mime {
    pub use mime_crate::*;
}
