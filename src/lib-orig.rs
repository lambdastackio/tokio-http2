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

#[macro_use] extern crate log;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate url;
extern crate rustc_serialize;
extern crate byteorder;

extern crate cookie;
extern crate futures;
extern crate futures_cpupool;
extern crate httparse;
extern crate net2;
extern crate time;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_tls;

//pub mod http2;
//pub mod hpack;

pub mod http;
pub mod uri;
pub mod version;
pub mod method;
pub mod error;
pub mod status;

pub type Body = Vec<u8>;
pub type ContentType = String;
pub type ContentLength = u64;
pub type Headers = Vec<(String, String)>;

pub use method::Method::{self, Get, Head, Post, Delete};
pub use status::StatusCode::{self, Ok, BadRequest, NotFound};
pub use uri::RequestUri;
pub use version::HttpVersion;
pub use error::{Result, Error};
pub use url::Url;
