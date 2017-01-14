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

use Method;

use router::handlers;
use router::path::RequestPath;
use Handler;
use super::builder::RouteBuilder;

/// Holds route information
pub struct Route {
    /// HTTP method to match
    pub method: Method,

    /// RequestPath to match
    pub path: RequestPath,

    /// Request handler
    ///
    /// This should be method that accepts Request and Response:
    ///
    pub handler: Handler
}

impl Route {
    pub fn options(path: &str) -> RouteBuilder {
        Route::from(Method::Options, path)
    }

    pub fn get(path: &str) -> RouteBuilder {
        Route::from(Method::Get, path)
    }

    pub fn post(path: &str) -> RouteBuilder {
        Route::from(Method::Post, path)
    }

    pub fn put(path: &str) -> RouteBuilder {
        Route::from(Method::Put, path)
    }

    pub fn delete(path: &str) -> RouteBuilder {
        Route::from(Method::Delete, path)
    }

    pub fn head(path: &str) -> RouteBuilder {
        Route::from(Method::Head, path)
    }

    pub fn trace(path: &str) -> RouteBuilder {
        Route::from(Method::Trace, path)
    }

    pub fn connect(path: &str) -> RouteBuilder {
        Route::from(Method::Connect, path)
    }

    pub fn patch(path: &str) -> RouteBuilder {
        Route::from(Method::Patch, path)
    }

    pub fn from(method: Method, path: &str) -> RouteBuilder {
        RouteBuilder::new(Route {
            method: method,
            path: RequestPath::new(path),
            .. Route::default()
        })
    }
}

impl Default for Route {
    fn default() -> Route {
        Route {
            method: Method::Get,
            path: RequestPath::new("/"),
            handler: handlers::not_implemented_handler
        }
    }
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Route {{method: {:?}, path: {:?}}}", self.method, self.path)
    }
}
