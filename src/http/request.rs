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

#![allow(dead_code)]

use std::{io, slice, str, fmt};
use std::fs::File;
use std::io::{Error, Read, BufReader};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::collections::hash_map::Entry::*;
use std::ops::DerefMut;
use std::cmp;
use std::str::FromStr;

use tokio_core::io::{EasyBuf, EasyBufMut};
use unicase::UniCase;
use httparse;
use url::form_urlencoded;

use multipart::server::{HttpRequest, Multipart, Entries, SaveResult};
use super::buffer::Buffer;
use Method;
use Handler;
use Router;
use Logger;

/// Just a reader - Created to enforce the Read trait and to leave the under lying EasyBuf alone.
#[derive(Clone)]
struct ReqReader {
    inner: EasyBuf,
    pos: usize,
    cap: usize,
}

impl ReqReader {
    fn new(inner: EasyBuf) -> ReqReader {
        ReqReader{ inner: inner.clone(), pos: 0, cap: inner.len() }
    }

    fn consume(&mut self, amt: usize) {
        self.pos = cmp::min(self.pos + amt, self.cap);
    }

    fn reset(&mut self) {
        self.pos = 0;
        self.cap = self.inner.len();
    }

    fn as_slice(&self) -> &[u8] {
        self.inner.as_slice()
    }
}

impl Read for ReqReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len  = cmp::min(buf.len(), self.cap - self.pos);
        buf[0..len].copy_from_slice(&self.inner.as_slice()[self.pos..self.pos + len]);
        self.consume(len);
        Ok(len)
    }
}

// NB: Slice is used so as to quickly extract portions of the buffer and to not have to use lifetimes.
// data: EasyBuf, - original data element below. Remove this comment later...

#[derive(Clone)]
pub struct Request {
    // Convenience - begin
    content_length: usize,
    content_type: String,
    content_type_metadata: String,
    host: String,
    // Convenience - end
    method: Slice,
    password: String,
    path: Slice,
    payload: Slice,
    query: Slice,
    request_line: String,
    scheme: String,
    uri: String,
    username: String,
    version: u8,
    remote_addr: Option<SocketAddr>,
    headers: Vec<(Slice, Slice)>,
    data: ReqReader,
    /// Handler associated with the specific request. If none then the application (server)
    /// will handle it in it's default routing.
    handler: Option<Handler>,
    /// Optional Logger associated with a given request
    pub logger: Option<Logger>,
    /// Base path is used to hold the actual base directory location on the server of the request.
    pub base_path: String,
}

type Slice = (usize, usize);

#[derive(Debug)]
pub struct RequestHeaders<'req> {
    pub headers: slice::Iter<'req, (Slice, Slice)>,
    req: &'req Request,
}

impl Read for Request {
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Result<usize, io::Error> {
        let len = try!(self.data.read(buf));
        Ok(len)
    }
}

impl Request {
    // Convenience methods
    pub fn content_length(&self) -> usize {
        self.content_length
    }

    pub fn content_type(&self) -> &str {
        match self.content_type.find(';') {
            Some(index) => &self.content_type[..index],
            None => &self.content_type,
        }
    }

    pub fn content_type_metadata(&self) -> Option<&str> {
        if self.content_type_metadata.is_empty() {
            None
        } else {
            Some(&self.content_type_metadata)
        }
    }

    pub fn content_type_all(&self) -> &str {
        &self.content_type
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn handler(&self) -> Option<Handler> {
        self.handler
    }

    pub fn method(&self) -> Method {
        let method = str::from_utf8(self.slice(&self.method)).unwrap();
        Method::from_str(method).unwrap_or(Method::Get)
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn path(&self) -> &str {
        str::from_utf8(self.slice(&self.path)).unwrap()
    }

    pub fn payload(&self) -> Option<&[u8]> {
        if self.payload.0 == 0 && self.payload.1 == 0 {
            None
        } else {
            Some(self.slice(&self.payload))
        }
    }

    pub fn query(&self) -> Option<HashMap<String, Vec<String>>> {
        if self.query.0 == 0 && self.query.1 == 0 {
            None
        } else {
            let data = str::from_utf8(self.slice(&self.query)).unwrap();
            Some(combine_duplicates(form_urlencoded::parse(data.as_bytes()).into_owned()))
        }
    }

    pub fn urldecode(&self, data: &[u8]) -> Option<HashMap<String, Vec<String>>> {
        if data.is_empty() {
            None
        } else {
            Some(combine_duplicates(form_urlencoded::parse(data).into_owned()))
        }
    }

    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    pub fn set_scheme(&mut self, scheme: &str) -> &str {
        self.scheme = scheme.to_string();
        // TODO: Update URI
        &self.scheme
    }

    pub fn set_remote_addr(&mut self, remote_addr: SocketAddr) {
        self.remote_addr = Some(remote_addr);
    }

    pub fn remote_addr(&mut self) -> Option<SocketAddr> {
        self.remote_addr
    }

    pub fn request_line(&self) -> &str {
        &self.request_line
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn user_agent(&self) -> Option<&str> {
        self.header("user-agent")
    }

    pub fn user_name(&self) -> &str {
        &self.username
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn header(&self, key: &str) -> Option<&str> {
        match self.headers().find(|&(k, v)| UniCase(k) == UniCase(key)) {
            Some((key, value)) => Some(str::from_utf8(value).unwrap_or("")),
            None => None
        }
    }

    pub fn headers(&self) -> RequestHeaders {
        RequestHeaders {
            headers: self.headers.iter(),
            req: self,
        }
    }

    // Extracts the data from the buffer at the given offset for the given length
    fn slice(&self, slice: &Slice) -> &[u8] {
        &self.data.as_slice()[slice.0..slice.1]
        // &self.data.slice()[slice.0..slice.1]
    }
}


// Extract header value using key. If not found or can't be converted to &str then None else the &str value.
fn header<'a>(req: &'a mut httparse::Request, key: &str) -> Option<&'a str> {
    let value: &str;

    for h in req.headers.iter() {
        if UniCase(h.name) == UniCase(key) {
            value = str::from_utf8(h.value).unwrap_or("");
            if value.is_empty() {
                return None;
            } else {
                return Some(value);
            }
        }
    }

    None
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<HTTP Request {} {}>", self.method(), self.path())
    }
}

fn combine_duplicates<I: Iterator<Item=(String, String)>>(collection: I) -> HashMap<String, Vec<String>> {
    let mut deduplicated: HashMap<String, Vec<String>> = HashMap::new();

    for (k, v) in collection {
        match deduplicated.entry(k) {
            // Already a Vec here, push onto it
            Occupied(entry) => { entry.into_mut().push(v); },

            // No value, create a one-element Vec.
            Vacant(entry) => { entry.insert(vec![v]); },
        };
    }

    deduplicated
}

/// Decode is a stand alone function since it creates the Request struct.
/// EasyBuf is from Tokio-core and is used for handling slices without having to create additional buffers.
/// Because of the slices, the methods used the begin and end parts of the Slice to determine where
/// to extract from the lower level EasyBuf.
pub fn decode(buf: &mut EasyBuf,
              remote_addr: Option<SocketAddr>,
              router: Option<Router>,
              logger: Option<Logger>,
              base_path: String)
              -> io::Result<Option<Request>> {
    let (content_length, content_type, content_type_metadata, handler, host, method, path, payload, query, request_line, scheme, uri, version, headers, amt) = {
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

        // println!("{:?}", String::from_utf8_lossy(buf.as_slice()));

        let toslice = |a: &[u8]| {
            let start = a.as_ptr() as usize - buf.as_slice().as_ptr() as usize;
            assert!(start < buf.len());
            (start, start + a.len())
        };

        let scheme = String::from("http");  // TODO: Hardcoded for now!
        let host = header(&mut r, "host").unwrap_or("").to_string();
        let content_type: String;

        match r.method {
            Some("POST") | Some("PUT") => content_type = header(&mut r, "content-type").unwrap_or("application/octet-stream").to_string(),
            Some(_) => content_type = header(&mut r, "accept").unwrap_or("text/plain").to_string(),
            None => content_type = "application/octet-stream".to_string(),
        }

        let mut content_type_metadata = String::new();

        match content_type.find(';') {
            Some(index) => {
                content_type_metadata = content_type[index+1..].trim().to_string();
            },
            None => {},
        }

        let content_length: usize = header(&mut r, "content-length").unwrap_or("0").parse::<usize>().unwrap_or(0);

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

        let mut handler: Option<Handler> = None;
        if router.is_some() {
            let m = Method::from_str(r.method.unwrap()).unwrap();
            let p = str::from_utf8(&buf.as_slice()[path.0..path.1]).unwrap_or("");
            handler = router.unwrap().find_handler_with_method_and_path(m, p);
        }

        let request_line = format!("{} {} HTTP/1.{}", r.method.unwrap(), r.path.unwrap_or(""), r.version.unwrap());

        (
         content_length,
         content_type,
         content_type_metadata,
         handler,
         host,
         method,
         path,
         payload,
         query,
         request_line,
         scheme,
         uri,
         r.version.unwrap(),
         r.headers
          .iter()
          .map(|h| (toslice(h.name.as_bytes()), toslice(h.value)))
          .collect(),
         amt
        )
    };

    let res = Request {
        base_path: base_path,
        content_length: content_length,
        content_type: content_type,
        content_type_metadata: content_type_metadata,
        host: host,
        method: method,
        password: "".to_string(),
        path: path,
        payload: payload,
        query: query,
        remote_addr: remote_addr,
        request_line: request_line,
        scheme: scheme,
        uri: uri,
        username: "".to_string(),
        version: version,
        headers: headers,
        data: ReqReader::new(buf.drain_to(amt)),
        handler: handler,
        logger: logger,
    };

    Ok(Some(res))
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

// Multipart
impl HttpRequest for Request {
    type Body = Self;

    fn multipart_boundary(&self) -> Option<&str> {
        const BOUNDARY: &'static str = "boundary=";

        match self.content_type_metadata() {
            Some(meta) => {
                let index = meta.find(BOUNDARY).unwrap_or(0) + BOUNDARY.len();
                Some(&meta[index..])
            },
            None => None
        }
    }

    fn body(self) -> Self::Body {
        self
    }
}
