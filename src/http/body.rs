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

use std::convert::From;
use std::sync::Arc;

use tokio_proto;
use http::Chunk;
use futures::{Poll, Stream};
use futures::sync::mpsc;

pub type TokioBody = tokio_proto::streaming::Body<Chunk, ::Error>;

/// A `Stream` for `Chunk`s used in requests and responses.
#[derive(Debug)]
pub struct Body(TokioBody);

impl Body {
    /// Return an empty body stream
    pub fn empty() -> Body {
        Body(TokioBody::empty())
    }

    /// Return a body stream with an associated sender half
    pub fn pair() -> (mpsc::Sender<Result<Chunk, ::Error>>, Body) {
        let (tx, rx) = TokioBody::pair();
        let rx = Body(rx);
        (tx, rx)
    }
}

impl Stream for Body {
    type Item = Chunk;
    type Error = ::Error;

    fn poll(&mut self) -> Poll<Option<Chunk>, ::Error> {
        self.0.poll()
    }
}

impl From<Body> for tokio_proto::streaming::Body<Chunk, ::Error> {
    fn from(b: Body) -> tokio_proto::streaming::Body<Chunk, ::Error> {
        b.0
    }
}

impl From<tokio_proto::streaming::Body<Chunk, ::Error>> for Body {
    fn from(tokio_body: tokio_proto::streaming::Body<Chunk, ::Error>) -> Body {
        Body(tokio_body)
    }
}

impl From<mpsc::Receiver<Result<Chunk, ::Error>>> for Body {
    fn from(src: mpsc::Receiver<Result<Chunk, ::Error>>) -> Body {
        Body(src.into())
    }
}

impl From<Chunk> for Body {
    fn from (chunk: Chunk) -> Body {
        Body(TokioBody::from(chunk))
    }
}

impl From<Vec<u8>> for Body {
    fn from (vec: Vec<u8>) -> Body {
        Body(TokioBody::from(Chunk::from(vec)))
    }
}

impl From<Arc<Vec<u8>>> for Body {
    fn from (vec: Arc<Vec<u8>>) -> Body {
        Body(TokioBody::from(Chunk::from(vec)))
    }
}

impl From<&'static [u8]> for Body {
    fn from (slice: &'static [u8]) -> Body {
        Body(TokioBody::from(Chunk::from(slice)))
    }
}

impl From<String> for Body {
    fn from (s: String) -> Body {
        Body(TokioBody::from(Chunk::from(s.into_bytes())))
    }
}

impl From<&'static str> for Body {
    fn from (slice: &'static str) -> Body {
        Body(TokioBody::from(Chunk::from(slice.as_bytes())))
    }
}

fn _assert_send() {
    fn _assert<T: Send>() {}

    _assert::<Body>();
    _assert::<Chunk>();
}
