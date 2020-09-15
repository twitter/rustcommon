// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::slice::Slice;
use rustcommon_histogram::{Counter, Histogram, HistogramError, Indexing};

use std::time::{Duration, Instant};

pub struct Heatmap<Value, Count> {
    pub(crate) slices: Vec<Histogram<Value, Count>>,
    pub(crate) current: usize,
    pub(crate) next_tick: Instant,
    pub(crate) resolution: Duration,
    pub(crate) summary: Histogram<Value, Count>,
}

impl<Value, Count> Heatmap<Value, Count>
where
    Value: Indexing,
    Count: Counter,
    u64: From<Value> + From<Count>,
{
    pub fn new(max: Value, precision: u8, windows: usize, resolution: Duration) -> Self {
        let mut slices = Vec::new();
        for _ in 0..windows {
            slices.push(Histogram::new(max, precision));
        }
        Self {
            slices,
            current: 0,
            next_tick: Instant::now() + resolution,
            resolution,
            summary: Histogram::new(max, precision),
        }
    }

    pub fn slices(&self) -> usize {
        self.slices.len()
    }

    pub fn buckets(&self) -> usize {
        self.summary.buckets()
    }

    pub fn increment(&mut self, time: Instant, value: Value, count: Count) {
        self.tick(time);
        self.slices[self.current].increment(value, count);
        self.summary.increment(value, count);
    }

    pub fn percentile(&mut self, percentile: f64) -> Result<Value, HistogramError> {
        self.tick(Instant::now());
        self.summary.percentile(percentile)
    }

    fn tick(&mut self, time: Instant) {
        while time >= self.next_tick {
            self.current += 1;
            if self.current >= self.slices.len() {
                self.current = 0;
            }
            self.next_tick += self.resolution;
            self.summary.sub_assign(&self.slices[self.current]);
            self.slices[self.current].clear();
        }
    }

    fn get_slice(&self, index: usize) -> Option<Slice<Value, Count>> {
        if let Some(histogram) = self.slices.get(index).map(|v| (*v).clone()) {
            let shift = if index > self.current {
                self.resolution
                    .mul_f64((self.slices.len() + self.current - index) as f64)
            } else {
                self.resolution.mul_f64((self.current - index) as f64)
            };
            Some(Slice {
                start: self.next_tick - shift - self.resolution,
                stop: self.next_tick - shift,
                histogram,
            })
        } else {
            None
        }
    }
}

pub struct Iter<'a, Value, Count>
where
    Value: Indexing,
    Count: Counter,
    u64: From<Value> + From<Count>,
{
    inner: &'a Heatmap<Value, Count>,
    index: usize,
    visited: usize,
}

impl<'a, Value, Count> Iter<'a, Value, Count>
where
    Value: Indexing,
    Count: Counter,
    u64: From<Value> + From<Count>,
{
    fn new(inner: &'a Heatmap<Value, Count>) -> Iter<'a, Value, Count> {
        let index = if inner.current < (inner.slices.len() - 1) {
            inner.current + 1
        } else {
            0
        };
        Iter {
            inner,
            index,
            visited: 0,
        }
    }
}

impl<'a, Value, Count> Iterator for Iter<'a, Value, Count>
where
    Value: Indexing,
    Count: Counter,
    u64: From<Value> + From<Count>,
{
    type Item = Slice<Value, Count>;

    fn next(&mut self) -> Option<Slice<Value, Count>> {
        if self.visited >= self.inner.slices.len() {
            None
        } else {
            let bucket = self.inner.get_slice(self.index);
            self.index += 1;
            if self.index >= self.inner.slices.len() {
                self.index = 0;
            }
            self.visited += 1;
            bucket
        }
    }
}

impl<'a, Value, Count> IntoIterator for &'a Heatmap<Value, Count>
where
    Value: Indexing,
    Count: Counter,
    u64: From<Value> + From<Count>,
{
    type Item = Slice<Value, Count>;
    type IntoIter = Iter<'a, Value, Count>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}
