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

use Request;
use Response;
use StatusCode;

/// Default handlers

pub fn default_404_handler(_: Request, _: String) -> Response {
    Response::new().with_header("Content-Length", "0").with_status(StatusCode::NotFound)
}

pub fn method_not_supported_handler(_: Request, _: String) -> Response {
    Response::new().with_header("Content-Length", "0").with_status(StatusCode::MethodNotAllowed)
}

pub fn internal_server_error_handler(_: Request, _: String) -> Response {
    Response::new().with_header("Content-Length", "0").with_status(StatusCode::InternalServerError)
}

pub fn not_implemented_handler(_: Request, _: String) -> Response {
    Response::new().with_header("Content-Length", "0").with_status(StatusCode::NotImplemented)
}
