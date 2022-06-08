// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! This crate provides an asynchronous logging backend that can direct logs to
//! one or more outputs.
//!
//! The core of this crate is the `AsyncLog` type, which is constructed using a
//! builder that is specific to your logging needs. After building the
//! `AsyncLog`, it can be registered as the global logger using the `start`
//! method. You will be left with a `Box<dyn Drain>` which should be
//! periodically flushed outside of any critical path. For example, in an admin
//! thread or dedicated logging thread.
//!
//! For logging to a single file, the `LogBuilder` type can be used to construct
//! an `AsyncLog` which has low overhead, but directs log messages to a single
//! `Output`.
//!
//! A `SamplingLogBuilder` can be used to construct an `AsyncLog` which will
//! filter the log messages using sampling before directing the log messages to
//! a single `Output`.
//!
//! A `MultiLogBuilder` can be used to construct an `AsyncLog` which routes log
//! messages based on the `target` metadata of the log `Record`. If there is an
//! `AsyncLog` registered for that specific `target`, then the log message will
//! be routed to that instance of `AsyncLog`. Log messages that do not match any
//! specific target will be routed to the default `AsyncLog` that has been added
//! to the `MultiLogBuilder`. If there is no default, messages that do not match
//! any specific target will be simply dropped.
//!
//! This combination of logging types allows us to compose a logging backend
//! which meets the application's needs. For example, you can use a local log
//! macro to set the target to some specific category and log those messages to
//! a file, while letting all other log messages pass to standard out. This
//! could allow splitting command/access/audit logs from the normal logging.

#[macro_use]
extern crate rustcommon_metrics;

pub use log::*;

mod format;
mod multi;
mod nop;
mod outputs;
mod sampling;
mod single;
mod traits;

pub use format::*;
pub use multi::*;
pub use nop::*;
pub use outputs::*;
pub use sampling::*;
pub use single::*;
pub use traits::*;

// use common::metrics::{static_metrics, Counter, Gauge};
use rustcommon_time::DateTime;
// use config::{DebugConfig, KlogConfig};
use mpmc::Queue;

pub(crate) type LogBuffer = Vec<u8>;

use rustcommon_metrics::{Counter, Gauge};

#[metric(name = "log_create", description = "logging targets initialized")]
pub static LOG_CREATE: Counter = Counter::new();
#[metric(
    name = "log_create_ex",
    description = "logging targets with initialization exceptions"
)]
pub static LOG_CREATE_EX: Counter = Counter::new();

#[metric(name = "log_destroy", description = "logging targets destroyed")]
pub static LOG_DESTROY: Counter = Counter::new();

#[metric(name = "log_curr", description = "current number of logging targets")]
pub static LOG_CURR: Gauge = Gauge::new();

#[metric(
    name = "log_open",
    description = "number of logging destinations which have been opened"
)]
pub static LOG_OPEN: Counter = Counter::new();

#[metric(
    name = "log_open_ex",
    description = "number of exceptions while opening logging destinations"
)]
pub static LOG_OPEN_EX: Counter = Counter::new();

#[metric(
    name = "log_write",
    description = "number of writes to all logging destinations"
)]
pub static LOG_WRITE: Counter = Counter::new();

#[metric(
    name = "log_write_byte",
    description = "total bytes written to all logging destinations"
)]
pub static LOG_WRITE_BYTE: Counter = Counter::new();

#[metric(
    name = "log_write_ex",
    description = "number of exceptions while writing to logging destinations"
)]
pub static LOG_WRITE_EX: Counter = Counter::new();

#[metric(
    name = "log_skip",
    description = "log messages skipped due to sampling"
)]
pub static LOG_SKIP: Counter = Counter::new();

#[metric(
    name = "log_drop",
    description = "log messages dropped due to full queues"
)]
pub static LOG_DROP: Counter = Counter::new();

#[metric(
    name = "log_drop_byte",
    description = "log bytes dropped due to full queues"
)]
pub static LOG_DROP_BYTE: Counter = Counter::new();

#[metric(
    name = "log_flush",
    description = "number of times logging destinations have been flushed"
)]
pub static LOG_FLUSH: Counter = Counter::new();

#[metric(
    name = "log_flush_ex",
    description = "number of exceptions while flushing logging destinations"
)]
pub static LOG_FLUSH_EX: Counter = Counter::new();

/// A type which implements an asynchronous logging backend.
pub struct AsyncLog {
    pub(crate) logger: Box<dyn Log>,
    pub(crate) drain: Box<dyn Drain>,
    pub(crate) level_filter: LevelFilter,
}

impl AsyncLog {
    /// Register the logger and return a type which implements `Drain`. It is
    /// up to the user to periodically call flush on the resulting drain.
    pub fn start(self) -> Box<dyn Drain> {
        let level_filter = self.level_filter;
        log::set_boxed_logger(self.logger)
            .map(|()| log::set_max_level(level_filter))
            .expect("failed to start logger");
        self.drain
    }
}

#[macro_export]
macro_rules! fatal {
    () => (
        error!();
        std::process::exit(1);
        );
    ($fmt:expr) => (
        error!($fmt);
        std::process::exit(1);
        );
    ($fmt:expr, $($arg:tt)*) => (
        error!($fmt, $($arg)*);
        std::process::exit(1);
        );
}
