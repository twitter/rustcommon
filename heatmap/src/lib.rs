// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub use rustcommon_atomics::*;
pub use rustcommon_histogram::{AtomicCounter, Counter, HistogramError, Indexing};
use rustcommon_histogram::{AtomicHistogram, Histogram};
use std::sync::RwLock;
use std::time::{Duration, Instant};

pub struct Heatmap<Value, Count> {
    slices: Vec<Histogram<Value, Count>>,
    current: usize,
    next_tick: Instant,
    resolution: Duration,
    summary: Histogram<Value, Count>,
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
}

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
    pub fn new(max: Value, precision: u8, windows: usize, resolution: Duration) -> Self {
        let mut slices = Vec::new();
        for _ in 0..windows {
            slices.push(AtomicHistogram::new(max, precision));
        }
        Self {
            slices,
            current: AtomicUsize::new(0),
            next_tick: RwLock::new(Instant::now() + resolution),
            resolution,
            summary: AtomicHistogram::new(max, precision),
        }
    }

    pub fn increment(&self, time: Instant, value: Value, count: <Count as Atomic>::Primitive) {
        self.tick(time);
        self.slices[self.current.load(Ordering::Relaxed)].increment(value, count);
        self.summary.increment(value, count);
    }

    pub fn percentile(&self, percentile: f64) -> Result<Value, HistogramError> {
        self.tick(Instant::now());
        self.summary.percentile(percentile)
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_out() {
        let mut heatmap = Heatmap::<u64, u64>::new(1_000_000, 2, 1000, Duration::from_millis(1));
        assert_eq!(heatmap.percentile(0.0), Err(HistogramError::Empty));
        heatmap.increment(Instant::now(), 1, 1);
        assert_eq!(heatmap.percentile(0.0), Ok(1));
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(heatmap.percentile(0.0), Ok(1));
        std::thread::sleep(Duration::from_millis(2000));
        assert_eq!(heatmap.percentile(0.0), Err(HistogramError::Empty));

        let heatmap =
            AtomicHeatmap::<u64, AtomicU64>::new(1_000_000, 2, 1000, Duration::from_millis(1));
        assert_eq!(heatmap.percentile(0.0), Err(HistogramError::Empty));
        heatmap.increment(Instant::now(), 1, 1);
        assert_eq!(heatmap.percentile(0.0), Ok(1));
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(heatmap.percentile(0.0), Ok(1));
        std::thread::sleep(Duration::from_millis(2000));
        assert_eq!(heatmap.percentile(0.0), Err(HistogramError::Empty));
    }
}
