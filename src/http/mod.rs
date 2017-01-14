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

use std::io;
use tokio_core::net::TcpStream;
use std::net::SocketAddr;

use tokio_proto::pipeline::ServerProto;
use tokio_core::io::{Io, Codec, Framed, EasyBuf};

pub use self::request::Request;
pub use self::response::Response;

mod date;
mod request;
mod response;
pub mod buffer;

pub struct HttpProto;

// codec here so as to create a Codec that can handle a remote_addr field.
impl HttpProto {
    fn codec(&self, remote_addr: SocketAddr) -> HttpCodec {
        HttpCodec{ remote_addr: Some(remote_addr) }
    }
}

impl ServerProto<TcpStream> for HttpProto {
    type Request = Request;
    type Response = Response;
    // type Error = io::Error;
    type Transport = Framed<TcpStream, HttpCodec>;
    type BindTransport = io::Result<Framed<TcpStream, HttpCodec>>;

    fn bind_transport(&self, io: TcpStream) -> io::Result<Framed<TcpStream, HttpCodec>> {
        let addr = io.peer_addr()?;
        Ok(io.framed(self.codec(addr)))
    }
}

// remote_addr is passed to the decode function to be added to the Request struct that eventually
// gets passed to the Service call method in the server application.
pub struct HttpCodec {
    remote_addr: Option<SocketAddr>,
}

impl Codec for HttpCodec {
    type In = Request;
    type Out = Response;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Request>> {
        request::decode(buf, self.remote_addr)
    }

    fn encode(&mut self, msg: Response, buf: &mut Vec<u8>) -> io::Result<()> {
        response::encode(msg, buf);
        Ok(())
    }
}

// Original shown here for example reference...

// pub struct Http;
//
// impl<T: Io + 'static> ServerProto<T> for Http {
//     type Request = Request;
//     type Response = Response;
//     type Error = io::Error;
//     type Transport = Framed<T, HttpCodec>;
//     type BindTransport = io::Result<Framed<T, HttpCodec>>;
//
//     fn bind_transport(&self, io: T) -> io::Result<Framed<T, HttpCodec>> {
//         Ok(io.framed(HttpCodec))
//     }
// }
//
// pub struct HttpCodec;
//
// impl Codec for HttpCodec {
//     type In = Request;
//     type Out = Response;
//
//     fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Request>> {
//         request::decode(buf)
//     }
//
//     fn encode(&mut self, msg: Response, buf: &mut Vec<u8>) -> io::Result<()> {
//         response::encode(msg, buf);
//         Ok(())
//     }
// }
