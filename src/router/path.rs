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

extern crate regex;
use self::regex::Regex;

/// Represents a path in HTTP sense (starting from `/`)
#[derive(Debug)]
pub struct RequestPath {
    pub matcher: Regex
}

impl RequestPath {
    /// Creates a new path.
    ///
    /// This method accepts regular expressions so you can
    /// write something like this:
    ///
    /// ```no_run
    /// RequestPath::new(r"/person/\d+");
    /// ```
    ///
    /// Note that you don't have to match beggining and end of the
    /// path using `^` and `$` - those are inserted for you automatically.
    pub fn new(path: &str) -> RequestPath {
        let mut regex = "^".to_string();
        regex.push_str(path);
        regex.push_str("$");
        RequestPath { matcher: Regex::new(&regex).unwrap() }
    }
}
