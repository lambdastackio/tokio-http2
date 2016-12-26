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

mod date;
mod request;
mod response;

use std::io;

use tokio_proto::pipeline::ServerProto;
use tokio_core::io::{Io, Codec, Framed, EasyBuf};

pub use self::request::Request;
pub use self::response::Response;

pub struct Http;

impl<T: Io + 'static> ServerProto<T> for Http {
    type Request = Request;
    type Response = Response;
    type Error = io::Error;
    type Transport = Framed<T, HttpCodec>;
    type BindTransport = io::Result<Framed<T, HttpCodec>>;

    fn bind_transport(&self, io: T) -> io::Result<Framed<T, HttpCodec>> {
        Ok(io.framed(HttpCodec))
    }
}

pub struct HttpCodec;

impl Codec for HttpCodec {
    type In = Request;
    type Out = Response;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Request>> {
        request::decode(buf)
    }

    fn encode(&mut self, msg: Response, buf: &mut Vec<u8>) -> io::Result<()> {
        response::encode(msg, buf);
        Ok(())
    }
}
