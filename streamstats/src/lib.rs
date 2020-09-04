// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_atomics::*;
use std::sync::RwLock;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum StreamstatsError {
    #[error("histogram contains no samples")]
    /// The histogram contains no samples.
    Empty,
    #[error("invalid percentile")]
    InvalidPercentile,
}

pub struct AtomicStreamstats<T>
where
    T: Atomic,
    <T as Atomic>::Primitive: Ord,
{
    buffer: Vec<T>,
    current: AtomicUsize,
    len: AtomicUsize,
    sorted: RwLock<Vec<<T as Atomic>::Primitive>>,
}

impl<T> AtomicStreamstats<T>
where
    T: Atomic + Default,
    <T as Atomic>::Primitive: Copy + Ord,
{
    /// Create a new struct which can hold up to `capacity` values in the
    /// buffer.
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        let sorted = RwLock::new(Vec::<<T as Atomic>::Primitive>::with_capacity(capacity));
        for _ in 0..capacity {
            buffer.push(Default::default());
        }
        Self {
            buffer,
            current: AtomicUsize::new(0),
            len: AtomicUsize::new(0),
            sorted,
        }
    }

    /// Insert a new value into the buffer.
    pub fn insert(&self, value: <T as Atomic>::Primitive) {
        let mut current = self.current.load(Ordering::Relaxed);
        loop {
            let next = if current < (self.buffer.len() - 1) {
                current + 1
            } else {
                0
            };
            let previous = self
                .current
                .compare_and_swap(current, next, Ordering::Relaxed);
            if previous == current {
                break;
            } else {
                current = previous;
            }
        }
        self.buffer[current].store(value, Ordering::Relaxed);
        if self.len.load(Ordering::Relaxed) < self.buffer.len() {
            self.len.fetch_add(1, Ordering::Relaxed);
        }
        self.sorted.write().unwrap().clear(); // resort required
    }

    fn values(&self) -> usize {
        let len = self.len.load(Ordering::Relaxed);
        if len < self.buffer.len() {
            len
        } else {
            self.buffer.len()
        }
    }

    /// Return the value closest to the specified percentile. Returns an error
    /// if the value is outside of the histogram range or if the histogram is
    /// empty. Percentile must be within the range 0.0 to 100.0
    pub fn percentile(
        &self,
        percentile: f64,
    ) -> Result<<T as Atomic>::Primitive, StreamstatsError> {
        if percentile < 0.0 || percentile > 100.0 {
            return Err(StreamstatsError::InvalidPercentile);
        }
        let sorted_len = { self.sorted.read().unwrap().len() };
        if sorted_len == 0 {
            let values = self.values();
            if values == 0 {
                return Err(StreamstatsError::Empty);
            } else {
                let mut sorted = self.sorted.write().unwrap();
                let values = self.values();
                for i in 0..values {
                    sorted.push(self.buffer[i].load(Ordering::Relaxed));
                }
                sorted.sort();
            }
        }
        let sorted = self.sorted.read().unwrap();
        if sorted.len() > 0 {
            if percentile == 0.0 {
                Ok(sorted[0])
            } else {
                let need = (percentile / 100.0 * sorted.len() as f64).ceil() as usize;
                Ok(sorted[need - 1])
            }
        } else {
            Err(StreamstatsError::Empty)
        }
    }

    pub fn clear(&mut self) {
        self.current.store(0, Ordering::Relaxed);
        self.len.store(0, Ordering::Relaxed);
        self.sorted.write().unwrap().clear();
    }
}

pub struct Streamstats<T> {
    buffer: Vec<T>,
    current: usize,
    oldest: usize,
    sorted: Vec<T>,
}

impl<T> Streamstats<T>
where
    T: Default + Copy + Ord,
{
    /// Create a new struct which can hold up to `capacity` values in the
    /// buffer.
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        let sorted = buffer.clone();
        for _ in 0..capacity {
            buffer.push(Default::default());
        }
        Self {
            buffer,
            current: 0,
            oldest: 0,
            sorted,
        }
    }

    /// Insert a new value into the buffer.
    pub fn insert(&mut self, value: T) {
        self.buffer[self.current] = value;
        self.current += 1;
        if self.current >= self.buffer.len() {
            self.current = 0;
        }
        if self.current == self.oldest {
            self.oldest += 1;
            if self.oldest >= self.buffer.len() {
                self.oldest = 0;
            }
        }
        self.sorted.clear(); // resort required
    }

    fn values(&self) -> usize {
        if self.current < self.oldest {
            (self.current + self.buffer.len()) - self.oldest
        } else if self.current == self.oldest {
            0
        } else {
            self.current - self.oldest
        }
    }

    /// Return the value closest to the specified percentile. Returns an error
    /// if the value is outside of the histogram range or if the histogram is
    /// empty. Percentile must be within the range 0.0 to 100.0
    pub fn percentile(&mut self, percentile: f64) -> Result<T, StreamstatsError> {
        if percentile < 0.0 || percentile > 100.0 {
            return Err(StreamstatsError::InvalidPercentile);
        }
        if self.sorted.len() == 0 {
            let values = self.values();
            if values == 0 {
                return Err(StreamstatsError::Empty);
            } else {
                if self.current > self.oldest {
                    for i in self.oldest..self.current {
                        self.sorted.push(self.buffer[i]);
                    }
                } else {
                    for i in self.oldest..self.buffer.len() {
                        self.sorted.push(self.buffer[i]);
                    }
                    for i in 0..self.current {
                        self.sorted.push(self.buffer[i]);
                    }
                }
                self.sorted.sort();
            }
        }
        if percentile == 0.0 {
            Ok(self.sorted[0])
        } else {
            let need = (percentile / 100.0 * self.sorted.len() as f64).ceil() as usize;
            Ok(self.sorted[need - 1])
        }
    }

    pub fn clear(&mut self) {
        self.oldest = self.current;
        self.sorted.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut streamstats = Streamstats::<u64>::new(1000);
        assert_eq!(streamstats.percentile(0.0), Err(StreamstatsError::Empty));
        streamstats.insert(1);
        assert_eq!(streamstats.percentile(0.0), Ok(1));
        streamstats.clear();
        assert_eq!(streamstats.percentile(0.0), Err(StreamstatsError::Empty));

        for i in 0..=10_000 {
            streamstats.insert(i);
            assert_eq!(streamstats.percentile(100.0), Ok(i));
        }

        let mut streamstats = AtomicStreamstats::<AtomicU64>::new(1000);
        assert_eq!(streamstats.percentile(0.0), Err(StreamstatsError::Empty));
        streamstats.insert(1);
        assert_eq!(streamstats.percentile(0.0), Ok(1));
        streamstats.clear();
        assert_eq!(streamstats.percentile(0.0), Err(StreamstatsError::Empty));

        for i in 0..=10_000 {
            streamstats.insert(i);
            assert_eq!(streamstats.percentile(100.0), Ok(i));
        }
    }
}
