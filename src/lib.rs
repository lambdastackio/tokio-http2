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
extern crate rustc_serialize;

pub mod http2;
pub mod hpack;
