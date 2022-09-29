// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Error;
use crate::*;
use core::sync::atomic::*;

use histogram::{Bucket, Histogram};

/// AtomicHeatmaps are concurrent datastructures which store counts for
/// timestamped values over a configured time range with individual histograms
/// arranged in a ring buffer. Increments occur in the most recent slice in the
/// buffer, unless they are newer than that slice may hold. When this happens,
/// old slices are cleared and reused. This configuration results in a fully
/// pre-allocated datastructure with concurrent read-write access.
pub struct Heatmap {
    slices: Vec<Histogram>,
    current: AtomicUsize,
    next_tick: AtomicInstant,
    resolution: Duration,
    summary: Histogram,
}

impl Heatmap {
    /// Create a new `AtomicHeatmap` which can store values up and including the
    /// `max` while maintaining precision across a wide range of values. The
    /// `precision` is expressed in the number of significant figures preserved.
    /// The heatmap will contain a histogram for each time step, specified by
    /// the resolution, necessary to represent the entire span of time stored
    /// within the heatmap. If the span is not evenly divisible by the
    /// resolution an additional window will be allocated and the true span will
    /// be slightly longer than the requested span. Smaller durations for the
    /// resolution cause more memory to be used, but a smaller batches of
    /// samples to age out at each time step.
    pub fn new(m: u32, r: u32, n: u32, span: Duration, resolution: Duration) -> Self {
        let mut slices = Vec::new();
        let mut true_span = Duration::from_nanos(0);
        while true_span < span {
            slices.push(Histogram::new(m, r, n));
            true_span += resolution;
        }
        slices.shrink_to_fit();
        let next_tick = AtomicInstant::now();
        next_tick.fetch_add(resolution, Ordering::Relaxed);
        Self {
            slices,
            current: AtomicUsize::new(0),
            next_tick,
            resolution,
            summary: Histogram::new(m, r, n),
        }
    }

    /// Returns the number of windows stored in the `AtomicHeatmap`
    pub fn windows(&self) -> usize {
        self.slices.len()
    }

    /// Returns the number of buckets stored within each `Histogram` in the
    /// `Heatmap`
    pub fn buckets(&self) -> usize {
        self.summary.buckets()
    }

    /// Increment a time-value pair by a specified count
    pub fn increment(&self, time: Instant, value: u64, count: u32) {
        self.tick(time);
        if let Some(slice) = self.slices.get(self.current.load(Ordering::Relaxed)) {
            let _ = slice.increment(value, count);
            let _ = self.summary.increment(value, count);
        }
    }

    /// Return the nearest value for the requested percentile (0.0 - 100.0)
    /// across the total range of samples retained in the `Heatmap`.
    ///
    /// Note: since the heatmap stores a distribution across a configured time
    /// span, sequential calls to fetch the percentile might result in different
    /// results even without concurrent writers. For instance, you may see a
    /// 90th percentile that is higher than the 100th percentile depending on
    /// the timing of calls to this function and the distribution of your data.
    ///
    /// Note: concurrent writes may also effect the value returned by this
    /// function. Users needing better consistency should ensure that other
    /// threads are not writing into the heatmap while this function is
    /// in-progress.
    pub fn percentile(&self, percentile: f64) -> Result<Bucket, Error> {
        self.tick(Instant::now());
        self.summary.percentile(percentile).map_err(Error::from)
    }

    // Internal function which handles reuse of older windows to store newer
    /// values.
    fn tick(&self, time: Instant) {
        loop {
            let next_tick = self.next_tick.load(Ordering::Relaxed);
            if time < next_tick {
                return;
            } else {
                self.next_tick.fetch_add(self.resolution, Ordering::Relaxed);
                self.current.fetch_add(1, Ordering::Relaxed);
                if self.current.load(Ordering::Relaxed) >= self.slices.len() {
                    self.current.store(0, Ordering::Relaxed);
                }
                let current = self.current.load(Ordering::Relaxed);
                if let Some(slice) = self.slices.get(current) {
                    let _ = self.summary.sub_assign(slice);
                    slice.clear();
                }
            }
        }
    }

    /// Internal function to return a `Window` from the `Heatmap`.
    fn get_slice(&self, index: usize) -> Option<Window> {
        if let Some(histogram) = self.slices.get(index) {
            let shift = if index > self.current.load(Ordering::Relaxed) {
                self.resolution.mul_f64(
                    (self.slices.len() + self.current.load(Ordering::Relaxed) - index) as f64,
                )
            } else {
                self.resolution
                    .mul_f64((self.current.load(Ordering::Relaxed) - index) as f64)
            };
            Some(Window {
                start: self.next_tick.load(Ordering::Relaxed) - shift - self.resolution,
                stop: self.next_tick.load(Ordering::Relaxed) - shift,
                histogram,
            })
        } else {
            None
        }
    }
}

impl Clone for Heatmap {
    fn clone(&self) -> Self {
        let slices = self.slices.clone();
        let summary = self.summary.clone();
        let resolution = self.resolution;
        let current = AtomicUsize::new(self.current.load(Ordering::Relaxed));
        let next_tick = AtomicInstant::new(self.next_tick.load(Ordering::Relaxed));

        Heatmap {
            slices,
            current,
            next_tick,
            resolution,
            summary,
        }
    }
}

pub struct Iter<'a> {
    inner: &'a Heatmap,
    index: usize,
    visited: usize,
}

impl<'a> Iter<'a> {
    fn new(inner: &'a Heatmap) -> Iter<'a> {
        let index = if inner.current.load(Ordering::Relaxed) < (inner.slices.len() - 1) {
            inner.current.load(Ordering::Relaxed) + 1
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

impl<'a> Iterator for Iter<'a> {
    type Item = Window<'a>;

    fn next(&mut self) -> Option<Window<'a>> {
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

impl<'a> IntoIterator for &'a Heatmap {
    type Item = Window<'a>;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_out() {
        let heatmap = Heatmap::new(0, 4, 20, Duration::from_secs(1), Duration::from_millis(1));
        assert_eq!(heatmap.percentile(0.0).map(|v| v.high()), Err(Error::Empty));
        heatmap.increment(Instant::now(), 1, 1);
        assert_eq!(heatmap.percentile(0.0).map(|v| v.high()), Ok(1));
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(heatmap.percentile(0.0).map(|v| v.high()), Ok(1));
        std::thread::sleep(std::time::Duration::from_millis(2000));
        assert_eq!(heatmap.percentile(0.0).map(|v| v.high()), Err(Error::Empty));
    }
}
