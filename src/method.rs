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

use std::fmt;
use std::str::FromStr;
use std::convert::AsRef;

use error::Error;
use self::Method::{Options, Get, Post, Put, Delete, Head, Trace, Connect, Patch,
                   Extension};


/// The Request Method (VERB)
///
/// Currently includes 8 variants representing the 8 methods defined in
/// [RFC 7230](https://tools.ietf.org/html/rfc7231#section-4.1), plus PATCH,
/// and an Extension variant for all extensions.
///
/// It may make sense to grow this to include all variants currently
/// registered with IANA, if they are at all common to use.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Method {
   /// OPTIONS
   Options,
   /// GET
   Get,
   /// POST
   Post,
   /// PUT
   Put,
   /// DELETE
   Delete,
   /// HEAD
   Head,
   /// TRACE
   Trace,
   /// CONNECT
   Connect,
   /// PATCH
   Patch,
   /// Method extensions. An example would be `let m = Extension("FOO".to_string())`.
   Extension(String)
}

impl AsRef<str> for Method {
   fn as_ref(&self) -> &str {
       match *self {
           Options => "OPTIONS",
           Get => "GET",
           Post => "POST",
           Put => "PUT",
           Delete => "DELETE",
           Head => "HEAD",
           Trace => "TRACE",
           Connect => "CONNECT",
           Patch => "PATCH",
           Extension(ref s) => s.as_ref()
       }
   }
}

impl Method {
   /// Whether a method is considered "safe", meaning the request is
   /// essentially read-only.
   ///
   /// See [the spec](https://tools.ietf.org/html/rfc7231#section-4.2.1)
   /// for more words.
   pub fn safe(&self) -> bool {
       match *self {
           Get | Head | Options | Trace => true,
           _ => false
       }
   }

   /// Whether a method is considered "idempotent", meaning the request has
   /// the same result if executed multiple times.
   ///
   /// See [the spec](https://tools.ietf.org/html/rfc7231#section-4.2.2) for
   /// more words.
   pub fn idempotent(&self) -> bool {
       if self.safe() {
           true
       } else {
           match *self {
               Put | Delete => true,
               _ => false
           }
       }
   }
}

impl FromStr for Method {
   type Err = Error;
   fn from_str(s: &str) -> Result<Method, Error> {
       if s == "" {
           Err(Error::Method)
       } else {
           Ok(match s {
               "OPTIONS" => Options,
               "GET" => Get,
               "POST" => Post,
               "PUT" => Put,
               "DELETE" => Delete,
               "HEAD" => Head,
               "TRACE" => Trace,
               "CONNECT" => Connect,
               "PATCH" => Patch,
               _ => Extension(s.to_owned())
           })
       }
   }
}

impl fmt::Display for Method {
   fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
       fmt.write_str(match *self {
           Options => "OPTIONS",
           Get => "GET",
           Post => "POST",
           Put => "PUT",
           Delete => "DELETE",
           Head => "HEAD",
           Trace => "TRACE",
           Connect => "CONNECT",
           Patch => "PATCH",
           Extension(ref s) => s.as_ref()
       })
   }
}

impl Default for Method {
   fn default() -> Method {
       Method::Get
   }
}
