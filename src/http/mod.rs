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

use tokio_tls::TlsAcceptorExt;
use native_tls::{Pkcs12, TlsAcceptor, TlsStream};

use Router;
use Logger;
use LoggerLevel;

pub use self::request::Request;
pub use self::response::Response;

mod date;
mod request;
mod response;
pub mod buffer;

/// Proto and Codec can have STATE so you can add features to these two and then pass them to
/// TcpServer.
#[derive(Default)]
pub struct HttpProto {
    pub logger: Option<Logger>,
    pub router: Option<Router>,
}

// codec here so as to create a Codec that can handle a remote_addr field.
impl HttpProto {
    fn codec(&self, remote_addr: SocketAddr, router: Option<Router>, logger: Option<Logger>) -> HttpCodec {
        HttpCodec{ request: None, remote_addr: Some(remote_addr), router: router, logger: logger }
    }
}

impl ServerProto<TcpStream> for HttpProto {
    type Request = Request;
    type Response = Response;
    type Transport = Framed<TcpStream, HttpCodec>;
    type BindTransport = io::Result<Framed<TcpStream, HttpCodec>>;

    fn bind_transport(&self, io: TcpStream) -> io::Result<Framed<TcpStream, HttpCodec>> {
        let addr = io.peer_addr()?;
        Ok(io.framed(self.codec(addr, self.router.clone(), self.logger.clone())))
    }
}

// remote_addr is passed to the decode function to be added to the Request struct that eventually
// gets passed to the Service call method in the server application.
pub struct HttpCodec {
    request: Option<Request>,
    remote_addr: Option<SocketAddr>,
    router: Option<Router>,
    logger: Option<Logger>,
}

impl Codec for HttpCodec {
    type In = Request;
    type Out = Response;

    /// HttpCodec::decode can be modified to fit whatever is needed.
    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Request>> {
        match request::decode(buf, self.remote_addr, self.router.clone(), self.logger.clone()) {
            Ok(req) => {
                match req {
                    Some(req) => {
                        self.request = Some(req.clone());
                        Ok(Some(req))
                    }
                    None => Ok(None)
                }
            }
            Err(e) => Err(e)
        }
    }

    fn encode(&mut self, msg: Response, buf: &mut Vec<u8>) -> io::Result<()> {
        response::encode(&msg, buf);
        if self.logger.is_some() {
            let logger = self.logger.clone().unwrap();
            let request = self.request.clone().unwrap();
            let referrer = "-"; //Check header
            let mut remote_addr = "-".to_string();
            match self.remote_addr {
                Some(val) => remote_addr = format!("{}", val),
                None => {},
            }
            logger.write(
                LoggerLevel::Info,
                format!("{} - \"{}\" {} {} \"{}\" \"{}\"",
                remote_addr,
                request.request_line(),
                msg.code,
                msg.content_length(),
                referrer,
                request.user_agent().unwrap_or("-")));
        }
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
