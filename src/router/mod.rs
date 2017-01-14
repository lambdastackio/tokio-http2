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

pub mod path;
pub mod route;
pub mod builder;
pub mod handlers;

use Request;
use Response;
use StatusCode;
use Method;
use Handler;

use self::path::RequestPath;
use self::route::route::Route;
use self::route::builder::RouteBuilder;
use self::builder::RouterBuilder;

pub type HttpResult<T> = Result<T, StatusCode>;

/// This is the one. The router.
#[derive(Debug)]
pub struct Router {
    routes: Vec<Route>
}

impl Router {
    /// Finds handler for given request.
    ///
    /// This method uses default error handlers.
    /// If the request does not match any route than default 404 handler is returned.
    /// If the request match some routes but http method does not match (used GET but routes are
    /// defined for POST) than default method not supported handler is returned.
    pub fn find_handler_with_defaults(&self, request: &Request) -> Handler {
        // if let Some(request_path) = request.path() {
        let request_path = request.path();
        let matching_routes = self.find_matching_routes(&request_path);
        match matching_routes.len() {
            x if x <= 0 => handlers::default_404_handler,
            _ => {
                self.find_for_method(&matching_routes, &request.method())
                    .unwrap_or(handlers::method_not_supported_handler)
            }
        }
        // } else {
        //     handlers::not_implemented_handler
        // }
    }

    /// Finds handler for given Hyper request.
    ///
    /// It returns handler if it's found or `StatusCode` for error.
    /// This method may return `NotFound`, `MethodNotAllowed` or `NotImplemented`
    /// status codes.
    pub fn find_handler(&self, request: &Request) -> HttpResult<Handler> {
        //if let AbsolutePath(request_path) = request.uri().clone() {
        // if let Some(request_path) = request.path() {
        let request_path = request.path();
        let matching_routes = self.find_matching_routes(&request_path);
        match matching_routes.len() {
            x if x <= 0 => Err(StatusCode::NotFound),
            _ => {
                self.find_for_method(&matching_routes, &request.method())
                    .map(|handler| Ok(handler))
                    .unwrap_or(Err(StatusCode::MethodNotAllowed))
            }
        }
        // } else {
        //     Err(StatusCode::NotImplemented)
        // }
    }

    /// Returns vector of `Route`s that match to given path.
    pub fn find_matching_routes(&self, request_path: &str) -> Vec<&Route> {
        self.routes.iter()
            .filter(|route| {
                route.path.matcher.is_match(&request_path)
            })
            .collect()
    }

    fn find_for_method(&self, routes: &Vec<&Route>, method: &Method) -> Option<Handler> {
        let method = method.clone();
        routes.iter()
            .find(|route| route.method == method)
            .map(|route| route.handler)
    }
}
