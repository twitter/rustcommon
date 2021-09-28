// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

use rustcommon_histogram::{Counter, Histogram, Indexing};

use rustcommon_time::{Duration, Instant};

/// Heatmaps are datastructures which store counts for timestamped values over a
/// configured time range with individual histograms arranged in a ring buffer.
/// Increments occur in the most recent slice in the buffer, unless they are
/// newer than that slice may hold. When this happens, old slices are cleared
/// and reused. This configuration results in a fully pre-allocated
/// datastructure.
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
    /// Create a new `Heatmap` which can store values up and including the `max`
    /// while maintaining precision across a wide range of values. The
    /// `precision` is expressed in the number of significant figures preserved.
    /// The heatmap will contain a histogram for each time step, specified by
    /// the resolution, necessary to represent the entire span of time stored
    /// within the heatmap. If the span is not evenly divisible by the
    /// resolution an additional window will be allocated and the true span will
    /// be slightly longer than the requested span. Smaller durations for the
    /// resolution cause more memory to be used, but a smaller batches of
    /// samples to age out at each time step.
    pub fn new(max: Value, precision: u8, span: Duration, resolution: Duration) -> Self {
        let mut slices = Vec::new();
        let mut true_span = Duration::new(0, 0);
        while true_span < span {
            slices.push(Histogram::new(max, precision));
            true_span += resolution;
        }
        Self {
            slices,
            current: 0,
            next_tick: Instant::now() + resolution,
            resolution,
            summary: Histogram::new(max, precision),
        }
    }

    /// Returns the number of windows stored in the `Heatmap`
    pub fn windows(&self) -> usize {
        self.slices.len()
    }

    /// Returns the number of buckets stored within each `Histogram` in the
    /// `Heatmap`
    pub fn buckets(&self) -> usize {
        self.summary.buckets()
    }

    /// Increment a time-value pair by a specified count
    pub fn increment(&mut self, time: Instant, value: Value, count: Count) {
        self.tick(time);
        if let Some(slice) = self.slices.get_mut(self.current) {
            slice.increment(value, count);
            self.summary.increment(value, count);
        }
    }

    /// Return the nearest value for the requested percentile (0.0 - 100.0)
    /// across the total range of samples retained in the `Heatmap`.
    ///
    /// Note: since the heatmap stores a distribution across a configured time
    /// span, sequential calls to fetch the percentile might result in different
    /// results. For instance, you may see a 90th percentile that is higher than
    /// the 100th percentile depending on the timing of calls to this function
    /// and the distribution of your data.
    pub fn percentile(&mut self, percentile: f64) -> Result<Value, HeatmapError> {
        self.tick(Instant::now());
        self.summary
            .percentile(percentile)
            .map_err(|e| HeatmapError::from(e))
    }

    /// Internal function which handles reuse of older windows to store newer
    /// values.
    fn tick(&mut self, time: Instant) {
        while time >= self.next_tick {
            self.current += 1;
            if self.current >= self.slices.len() {
                self.current = 0;
            }
            self.next_tick += self.resolution;
            if let Some(slice) = self.slices.get_mut(self.current) {
                self.summary.sub_assign(slice);
                slice.clear();
            }
        }
    }

    /// Internal function to return a `Window` from the `Heatmap`.
    fn get_slice(&self, index: usize) -> Option<Window<Value, Count>> {
        if let Some(histogram) = self.slices.get(index).map(|v| (*v).clone()) {
            let shift = if index > self.current {
                self.resolution
                    .mul_f64((self.slices.len() + self.current - index) as f64)
            } else {
                self.resolution.mul_f64((self.current - index) as f64)
            };
            Some(Window {
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
    type Item = Window<Value, Count>;

    fn next(&mut self) -> Option<Window<Value, Count>> {
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
    type Item = Window<Value, Count>;
    type IntoIter = Iter<'a, Value, Count>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}
