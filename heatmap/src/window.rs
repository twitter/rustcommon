// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use histogram::Histogram;
use rustcommon_time::*;

pub struct Window<'a> {
    pub(crate) start: Instant<Nanoseconds<u64>>,
    pub(crate) stop: Instant<Nanoseconds<u64>>,
    pub(crate) histogram: &'a Histogram,
}

impl<'a> Window<'a> {
    pub fn start(&self) -> Instant<Nanoseconds<u64>> {
        self.start
    }

    pub fn stop(&self) -> Instant<Nanoseconds<u64>> {
        self.stop
    }

    pub fn histogram(&self) -> &'a Histogram {
        &self.histogram
    }
}
