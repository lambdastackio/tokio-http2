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
use std::fs::File;
use std::io::{Error, Read};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::collections::hash_map::Entry::*;
use std::ops::DerefMut;
use std::cmp;

use tokio_core::io::{EasyBuf, EasyBufMut};
use unicase::UniCase;
use httparse;
use url::form_urlencoded;

use multipart::server::{HttpRequest, Multipart, Entries, SaveResult};
use super::buffer::Buffer;

// use version::HttpVersion;

// NB: Slice is used so as to quickly extract portions of the buffer and to not have to use lifetimes.

#[derive(Clone)]
pub struct Request {
    // Convenience
    content_length: usize,
    content_type: String,
    content_type_metadata: String,
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
    pub remote_addr: Option<SocketAddr>,
    // TODO: use a small vec to avoid this unconditional allocation
    headers: Vec<(Slice, Slice)>,
    data: EasyBuf,
    buffer: Buffer,
    // offset: usize,
    // buf_reader: Option<BufReader>,
}

type Slice = (usize, usize);

#[derive(Debug)]
pub struct RequestHeaders<'req> {
    pub headers: slice::Iter<'req, (Slice, Slice)>,
    req: &'req Request,
}

impl Read for Request {
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Result<usize, io::Error> {
        let len = try!(self.buffer.bytes().read(buf));
        println!("{:?}", len);
        // let data_len = self.data.len();
        // let mut data = self.data.get_mut();
        // let ref mut buffer: Vec<u8> = *data.deref_mut();
        // println!("{:?}", buf.len());
        // println!("{:?}", buffer.len());
        // let buf_len = buf.len();
        // let len = cmp::min(buf_len, buffer.len());
        // println!("{:?}", len);
        // buf[..len].copy_from_slice(&buffer[..len]);

        // buf[..len].copy_from_slice(&buffer[self.offset..len]);
        // self.offset += len;
        // if self.offset >= data_len {
        //     self.offset = 0;
        // }
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

    // pub fn content_type_metadata(&self) -> Option<Vec<&str>> {
    //     match self.content_type.find(';') {
    //         Some(index) => {
    //             let metadata: Vec<&str> = self.content_type[index+1..].split_terminator(';').map(|value| value.trim()).collect();
    //             Some(metadata)
    //         },
    //         None => None,
    //     }
    // }

    pub fn content_type_all(&self) -> &str {
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

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn username(&self) -> &str {
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
    }
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

pub fn decode(buf: &mut EasyBuf, remote_addr: Option<SocketAddr>) -> io::Result<Option<Request>> {
    let (buffer, content_length, content_type, content_type_metadata, host, method, path, payload, query, scheme, uri, version, headers, amt) = {
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

        let scheme = String::from("http");  // Hardcoded for now!
        let host = r.header("host").unwrap_or("").to_string();
        let content_type: String;
        match r.method {
            Some("POST") | Some("PUT") => content_type = r.header("content-type").unwrap_or("application/octet-stream").to_string(),
            Some(_) => content_type = r.header("accept").unwrap_or("text/plain").to_string(),
            None => content_type = "application/octet-stream".to_string(),
        }
        // let mut content_type_metadata: HashMap<&str, &str> = HashMap::new();
        // match content_type.find(';') {
        //     Some(index) => {
        //         let metadata: Vec<&str> = content_type[index+1..].split_terminator(';').map(|value| value.trim()).collect();
        //         for meta in metadata {
        //             let meta_split: Vec<&str> = meta.split('=').collect();
        //             if !meta_split.is_empty() {
        //                 content_type_metadata.insert(meta_split[0], meta_split[1]);
        //             }
        //         }
        //     },
        //     None => {},
        // }

        let mut content_type_metadata = String::new();
        match content_type.find(';') {
            Some(index) => {
                content_type_metadata = content_type[index+1..].trim().to_string();
            },
            None => {},
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

        let mut buffer: Buffer = Buffer::new();
        buffer.write(&buf.as_slice()[payload.0..payload.1]);

        (buffer,
         content_length,
         content_type,
         content_type_metadata, // if content_type_metadata.is_empty() {None} else {Some(content_type_metadata)},
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
        // buf_reader: buf_reader,
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
        scheme: scheme,
        uri: uri,
        username: "".to_string(),
        version: version,
        headers: headers,
        data: buf.drain_to(amt),
        buffer: buffer,
        // offset: 0,
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

// Multipart

// impl<'r> HttpRequest for &'r mut Request {
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

    fn body(self) -> Self {
        self
    }
}
