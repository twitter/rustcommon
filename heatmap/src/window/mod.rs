// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_histogram::Histogram;
use rustcommon_time::*;

pub struct Window<Value, Count> {
    pub(crate) start: Instant<Nanoseconds<u64>>,
    pub(crate) stop: Instant<Nanoseconds<u64>>,
    pub(crate) histogram: Histogram<Value, Count>,
}

impl<Value, Count> Window<Value, Count> {
    pub fn start(&self) -> Instant<Nanoseconds<u64>> {
        self.start
    }

    pub fn stop(&self) -> Instant<Nanoseconds<u64>> {
        self.stop
    }

    pub fn histogram(&self) -> &Histogram<Value, Count> {
        &self.histogram
    }
}
