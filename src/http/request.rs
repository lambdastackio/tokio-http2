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

use std::{io, slice, str, fmt};
use std::net::SocketAddr;

use tokio_core::io::EasyBuf;
// use uri::RequestUri;
use httparse;

use version::HttpVersion;

// NB: Slice is used so as to quickly extract portions of the buffer and to not have to use lifetimes.

pub struct Request {
    // Convenience
    content_length: usize,
    content_type: String,
    host: String,

    method: Slice,
    password: String,
    path: Slice,
    payload: Slice,
    query: Slice,
    scheme: String,
    uri: String,
    username: String,
    version: u8,
    // remote_addr: SocketAddr,
    // TODO: use a small vec to avoid this unconditional allocation
    headers: Vec<(Slice, Slice)>,
    data: EasyBuf,
}

type Slice = (usize, usize);

pub struct RequestHeaders<'req> {
    headers: slice::Iter<'req, (Slice, Slice)>,
    req: &'req Request,
}

impl Request {
    // Convenience methods
    pub fn content_length(&self) -> usize {
        self.content_length
    }

    /// For non POST, PUT methods this represents `Accept` header.
    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn method(&self) -> &str {
        str::from_utf8(self.slice(&self.method)).unwrap()
    }

    /// The remote socket address of this request
    // #[inline]
    // pub fn remote_addr(&self) -> &SocketAddr { &self.remote_addr }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn path(&self) -> &str {
        str::from_utf8(self.slice(&self.path)).unwrap()
    }

    pub fn payload(&self) -> Option<&[u8]> {
        // str::from_utf8(self.slice(&self.payload)).unwrap()
        if self.payload.0 == 0 && self.payload.1 == 0 {
            None
        } else {
            Some(self.slice(&self.payload))
        }
    }

    pub fn query(&self) -> &str {
        str::from_utf8(self.slice(&self.query)).unwrap()
    }

    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    pub fn set_scheme(&mut self, scheme: &str) -> &str {
        self.scheme = scheme.to_string();
        // TODO: Update URI
        &self.scheme
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    /// The headers of the incoming request.
    // #[inline]
    // pub fn headers(&self) -> &Headers { &self.headers }

    pub fn headers(&self) -> RequestHeaders {
        RequestHeaders {
            headers: self.headers.iter(),
            req: self,
        }
    }

    // Extracts the data from the buffer at the given offset for the given length
    fn slice(&self, slice: &Slice) -> &[u8] {
        &self.data.as_slice()[slice.0..slice.1]
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<HTTP Request {} {}>", self.method(), self.path())
    }
}

pub fn decode(buf: &mut EasyBuf) -> io::Result<Option<Request>> {
    let (content_length, content_type, host, method, path, payload, query, scheme, uri, version, headers, amt) = {
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut r = httparse::Request::new(&mut headers);
        let status = try!(r.parse(buf.as_slice()).map_err(|e| {
            let msg = format!("failed to parse http request: {:?}", e);
            io::Error::new(io::ErrorKind::Other, msg)
        }));

        let mut amt = match status {
            httparse::Status::Complete(amt) => amt,
            httparse::Status::Partial => return Ok(None),
        };

        println!("{:?}", String::from_utf8_lossy(buf.as_slice()));

        let toslice = |a: &[u8]| {
            let start = a.as_ptr() as usize - buf.as_slice().as_ptr() as usize;
            assert!(start < buf.len());
            (start, start + a.len())
        };

        let scheme = String::from("http");  // Hardcoded for now!
        let host = r.header("host").unwrap_or("").to_string();
        let mut content_type = String::from("");
        match r.method {
            Some("POST") | Some("PUT") => content_type = r.header("content-type").unwrap_or("application/octet-stream").to_string(),
            Some(_) => content_type = r.header("accept").unwrap_or("text/plain").to_string(),
            None => content_type = "application/octet-stream".to_string(),
        }
        let content_length: usize = r.header("content-length").unwrap_or("0").parse::<usize>().unwrap_or(0);
        // Adjust `amt` to also include payload
        amt += content_length;

        let method = toslice(r.method.unwrap().as_bytes());
        let uri = toslice(r.path.unwrap().as_bytes());
        let uri_str = r.path.unwrap();
        let query: Slice;
        let path: Slice;
        let payload: Slice = if content_length > 0 {((amt as u64 - content_length as u64) as usize, amt)} else {(0,0)};

        // NB: Parse out username and password for basic auth later

        if let Some(index) = uri_str.find('?') {
            path = (uri.0, uri.0 + index);
            query = (path.1 + 1, uri.1);
        } else {
            path = (uri.0, uri.1);
            query = (0, 0);
        }

        let uri = format!("{}://{}{}", scheme, host, uri_str);

        (content_length,
         content_type,
         host,
         method,
         path,
         payload,
         query,
         scheme,
         uri,
         r.version.unwrap(),
         r.headers
          .iter()
          .map(|h| (toslice(h.name.as_bytes()), toslice(h.value)))
          .collect(),
         amt)
    };

    Ok(Request {
        content_length: content_length,
        content_type: content_type,
        host: host,
        method: method,
        password: "".to_string(),
        path: path,
        payload: payload,
        query: query,
        scheme: scheme,
        uri: uri,
        username: "".to_string(),
        version: version,
        headers: headers,
        data: buf.drain_to(amt),
    }.into())
}

impl<'req> Iterator for RequestHeaders<'req> {
    type Item = (&'req str, &'req [u8]);

    fn next(&mut self) -> Option<(&'req str, &'req [u8])> {
        self.headers.next().map(|&(ref a, ref b)| {
            let a = self.req.slice(a);
            let b = self.req.slice(b);
            (str::from_utf8(a).unwrap(), b)
        })
    }
}
