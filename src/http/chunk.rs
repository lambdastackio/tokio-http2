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

use std::borrow::Borrow;
use std::fmt;
use std::sync::Arc;

/// A piece of a message body.
pub struct Chunk(Inner);

enum Inner {
    Owned(Vec<u8>),
    Referenced(Arc<Vec<u8>>),
    Static(&'static [u8]),
}

impl From<Vec<u8>> for Chunk {
    #[inline]
    fn from(v: Vec<u8>) -> Chunk {
        Chunk(Inner::Owned(v))
    }
}

impl From<Arc<Vec<u8>>> for Chunk {
    #[inline]
    fn from(v: Arc<Vec<u8>>) -> Chunk {
        Chunk(Inner::Referenced(v))
    }
}

impl From<&'static [u8]> for Chunk {
    #[inline]
    fn from(slice: &'static [u8]) -> Chunk {
        Chunk(Inner::Static(slice))
    }
}

impl ::std::ops::Deref for Chunk {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<[u8]> for Chunk {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        match self.0 {
            Inner::Owned(ref vec) => vec,
            Inner::Referenced(ref vec) => {
                let v: &Vec<u8> = vec.borrow();
                v.as_slice()
            }
            Inner::Static(slice) => slice,
        }
    }
}

impl fmt::Debug for Chunk {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_ref(), f)
    }
}
