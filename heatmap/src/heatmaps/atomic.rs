// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

use rustcommon_atomics::*;
use rustcommon_histogram::{AtomicCounter, AtomicHistogram, Counter, Indexing};

use std::sync::RwLock;
use std::time::{Duration, Instant};

/// AtomicHeatmaps are concurrent datastructures which store counts for
/// timestamped values over a configured time range with individual histograms
/// arranged in a ring buffer. Increments occur in the most recent slice in the
/// buffer, unless they are newer than that slice may hold. When this happens,
/// old slices are cleared and reused. This configuration results in a fully
/// pre-allocated datastructure with concurrent read-write access.
pub struct AtomicHeatmap<Value, Count> {
    slices: Vec<AtomicHistogram<Value, Count>>,
    current: AtomicUsize,
    next_tick: RwLock<Instant>,
    resolution: Duration,
    summary: AtomicHistogram<Value, Count>,
}

impl<Value, Count> AtomicHeatmap<Value, Count>
where
    Value: Indexing,
    Count: AtomicCounter + Default,
    u64: From<Value> + From<<Count as Atomic>::Primitive>,
    <Count as Atomic>::Primitive: Copy,
{
    /// Create a new `AtomicHeatmap` which can store values up and including the
    /// `max` while maintaining precision across a wide range of values. The
    /// `precision` is expressed in the number of significant figures preserved.
    /// The heatmap will store a histogram for each of the `windows` where each
    /// window will consist of a duration specified as the `resolution`. The
    /// combination of the number of windows and resolution places bounds on the
    /// overall span of time maintained within the heatmap as well as how much
    /// of the heatmap will be cleared when windows age-out.
    pub fn new(max: Value, precision: u8, windows: usize, resolution: Duration) -> Self {
        let mut slices = Vec::new();
        for _ in 0..windows {
            slices.push(AtomicHistogram::new(max, precision));
        }
        slices.shrink_to_fit();
        Self {
            slices,
            current: AtomicUsize::new(0),
            next_tick: RwLock::new(Instant::now() + resolution),
            resolution,
            summary: AtomicHistogram::new(max, precision),
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
    pub fn increment(&self, time: Instant, value: Value, count: <Count as Atomic>::Primitive) {
        self.tick(time);
        self.slices[self.current.load(Ordering::Relaxed)].increment(value, count);
        self.summary.increment(value, count);
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
    pub fn percentile(&self, percentile: f64) -> Result<Value, HeatmapError> {
        self.tick(Instant::now());
        self.summary.percentile(percentile).map_err(|e| HeatmapError::from(e))
    }

    // Internal function which handles reuse of older windows to store newer
    /// values.
    fn tick(&self, time: Instant) {
        loop {
            if time < *self.next_tick.read().unwrap() {
                return;
            } else {
                let mut next_tick = self.next_tick.write().unwrap();
                *next_tick += self.resolution;
                self.current.fetch_add(1, Ordering::Relaxed);
                if self.current.load(Ordering::Relaxed) >= self.slices.len() {
                    self.current.store(0, Ordering::Relaxed);
                }
                let current = self.current.load(Ordering::Relaxed);
                self.summary.sub_assign(&self.slices[current]);
                self.slices[current].clear();
            }
        }
    }

    /// Performs a `Relaxed` load of the current `AtomicHeatmap` allocating and
    /// returning a non-atomic `Heatmap`.
    ///
    /// Note: data may be inconsistent if there are concurrent writes happening
    /// while the load operation is performed. Users needing better consistency
    /// should ensure that other threads are not writing into the heatmap while
    /// this operation is in-progress.
    pub fn load(&self) -> Heatmap<Value, <Count as Atomic>::Primitive>
    where
        Value: Copy + std::ops::Sub<Output = Value>,
        <Count as Atomic>::Primitive: Counter,
    {
        let mut result = Heatmap {
            slices: Vec::with_capacity(self.slices.len()),
            current: self.current.load(Ordering::Relaxed),
            next_tick: self.next_tick.read().unwrap().clone(),
            resolution: self.resolution,
            summary: self.summary.load(),
        };
        for slice in &self.slices {
            result.slices.push(slice.load());
        }
        result.slices.shrink_to_fit();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_out() {
        let mut heatmap = Heatmap::<u64, u64>::new(1_000_000, 2, 1000, Duration::from_millis(1));
        assert_eq!(heatmap.percentile(0.0), Err(HeatmapError::Empty));
        heatmap.increment(Instant::now(), 1, 1);
        assert_eq!(heatmap.percentile(0.0), Ok(1));
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(heatmap.percentile(0.0), Ok(1));
        std::thread::sleep(Duration::from_millis(2000));
        assert_eq!(heatmap.percentile(0.0), Err(HeatmapError::Empty));

        let heatmap =
            AtomicHeatmap::<u64, AtomicU64>::new(1_000_000, 2, 1000, Duration::from_millis(1));
        assert_eq!(heatmap.percentile(0.0), Err(HeatmapError::Empty));
        heatmap.increment(Instant::now(), 1, 1);
        assert_eq!(heatmap.percentile(0.0), Ok(1));
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(heatmap.percentile(0.0), Ok(1));
        std::thread::sleep(Duration::from_millis(2000));
        assert_eq!(heatmap.percentile(0.0), Err(HeatmapError::Empty));
    }
}
