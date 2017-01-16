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

//! Logger
//!

use std::fmt;
use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use std::io;

use chrono::*;
use libc;
// use isatty::{stdout_isatty, stderr_isatty};

use slog;
use slog_term;
use slog_syslog;
use slog_stream;
use slog::{Drain, DrainExt};
use slog_syslog::Streamer3164;

#[derive(Debug, Clone)]
pub struct Logger {
    pub logger: slog::Logger,
    // level:
}

/// Custom format for Log File
struct LoggerFormat;

pub enum LoggerLevel {
    Debug,
    Error,
    Info,
    Warn,
}

// Maybe expose the slog?
// Maybe allow init options to be passed in?

#[cfg(target_os = "linux")]
fn logger(path: Option<&str>) -> slog::Logger {
    if path.is_some() {
        let file = OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(path.unwrap()).unwrap();
        // let stream = slog_stream::stream(file, LoggerFormat);//.fuse();
        //
        // if tty {
        //     let term = slog_term::streamer().build();//.fuse();
        //     slog::Logger::root(slog::duplicate(stream, term).fuse(), o!())
        // } else {
        //     slog::Logger::root(stream.fuse(), o!())
        // }

        let stream = slog_stream::stream(file, LoggerFormat);//.fuse();
        let term = slog_term::streamer().build();//.fuse();
        // slog::Logger::root(stream, o!())
        // slog::Logger::root(slog::Duplicate::new(
        //                    slog::LevelFilter::new(stream, slog::Level::Info),
        //                    slog::LevelFilter::new(term, slog::Level::Info),
        //                ).fuse(), o!())
        slog::Logger::root(slog::duplicate(stream, term).fuse(), o!())
    } else {
        let stream = slog_syslog::unix_3164(slog_syslog::Facility::LOG_DAEMON).fuse();
        slog::Logger::root(stream, o!())
    }
}

#[cfg(not(target_os = "linux"))]
fn logger(path: Option<&str>) -> slog::Logger {
    // let tty = stdout_isatty();

    if path.is_some() {
        let file = OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(path.unwrap()).unwrap();
        let stream = slog_stream::stream(file, LoggerFormat);//.fuse();

        let tty = unsafe { libc::isatty(libc::STDOUT_FILENO as i32) } != 0;

        if tty {
            let term = slog_term::streamer().build();//.fuse();
            slog::Logger::root(slog::duplicate(stream, term).fuse(), o!())
        } else {
            slog::Logger::root(stream.fuse(), o!())
        }
    } else {
        // Assumes tty is available
        let stream = slog_term::streamer().build().fuse();
        slog::Logger::root(stream, o!())
    }
}

// Override formatting here for log files so put level info and/or time here or in Logger::write
impl slog_stream::Format for LoggerFormat {
    fn format(&self,
              io: &mut io::Write,
              rinfo: &slog::Record,
              _logger_values: &slog::OwnedKeyValueList)
              -> io::Result<()> {
        let msg = format!("[{}] {}\n", UTC::now(), rinfo.msg());
        let _ = try!(io.write_all(msg.as_bytes()));
        Ok(())
    }
}

impl Logger {
    pub fn new(path: Option<&str>) -> Logger {
        let root_logger = logger(path);

        Logger{ logger: root_logger }
    }

    pub fn write(&self, logger_level: LoggerLevel, line: String) {
        match logger_level {
            LoggerLevel::Error => error!(self.logger, line),
            LoggerLevel::Debug => debug!(self.logger, line),
            LoggerLevel::Warn => warn!(self.logger, line),
            _ => info!(self.logger, line),
        }
    }
}
