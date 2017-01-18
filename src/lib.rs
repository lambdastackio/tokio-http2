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

// Multipart portion brought internal due to openssl issue in Linux.
// Copyright 2016 `multipart` Crate Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]

// NOTE: This attribute only needs to be set once.
#![doc(html_logo_url = "https://lambdastackio.github.io/static/images/lambdastack-200x200.png",
       html_favicon_url = "https://lambdastackio.github.io/static/images/favicon.ico",
       html_root_url = "https://lambdastackio.github.io/tokio-http2/tokio_http2/index.html")]

//! HTTP/1.1:
//! This library provides an HTTP library built on Futures and the Tokio Project for Async I/O.
//! This version supports Pipelining for HTTP/1.1.
//!
//! HTTP/2: (Interface being released soon)
//! This library supplies the required modules to implement HTTP/2 which includes the HPACK header
//! compression that includes the Huffman encoding/decoding features. This version will support
//! Multiplexing which is required for HTTP/2.

#[macro_use] extern crate log;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate url;
#[macro_use] extern crate slog;
extern crate pretty_env_logger;
extern crate slog_term;
extern crate slog_json;
extern crate slog_stream;
extern crate slog_syslog;
extern crate unicase;
extern crate rustc_serialize;
extern crate byteorder;
extern crate mime;
extern crate mime_guess;
extern crate rand;
extern crate tempdir;

// extern crate multipart;

// extern crate cookie;
extern crate futures;
extern crate futures_cpupool;
extern crate httparse;
extern crate net2;
extern crate time;
extern crate chrono;
extern crate libc;
extern crate native_tls;

extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_tls;

// For now...
// pub mod http2;
// pub mod hpack;

pub mod http;
pub mod version;
pub mod error;
pub mod status;
pub mod method;
pub mod router;
pub mod logger;

pub use status::StatusCode::{self, Ok, BadRequest, NotFound};
pub use version::HttpVersion;
pub use error::{Result, Error};
pub use url::Url;
pub use method::Method;
pub use http::{Request, Response};
pub use router::route::route::Route;
pub use router::Router;
pub use router::builder::RouterBuilder;
pub use logger::{Logger, LoggerLevel};

pub type Body = Vec<u8>;
pub type ContentType = String;
pub type ContentLength = u64;
pub type Headers = Vec<(String, String)>;

// NOTE: May want to add an `enum` of options as the second param for a Handler that allows for flexible options
// pub enum Options...
// (i.e., pub type Handler = fn(Request, Options) -> Response;)
pub type Handler = fn(Request, String) -> Response;


// Section below from multipart crate
use rand::Rng;

#[macro_export]
macro_rules! chain_result {
    ($first_expr:expr, $($try_expr:expr),*) => (
        $first_expr $(.and_then(|_| $try_expr))*
    );
    ($first_expr:expr, $($($arg:ident),+ -> $try_expr:expr),*) => (
        $first_expr $(.and_then(|$($arg),+| $try_expr))*
    );
}

// pub mod client;
pub mod server;

fn random_alphanumeric(len: usize) -> String {
    rand::thread_rng().gen_ascii_chars().take(len).collect()
}
