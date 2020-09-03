// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub trait Value: Default + Copy + Ord {}

impl Value for u64 {}
impl Value for u32 {}
impl Value for u16 {}
impl Value for u8 {}
impl Value for usize {}

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum StreamstatsError {
    #[error("histogram contains no samples")]
    /// The histogram contains no samples.
    Empty,
    #[error("invalid percentile")]
    InvalidPercentile,
}

pub struct Streamstats<T> {
    buffer: Vec<T>,
    current: usize,
    oldest: usize,
    sorted: Vec<T>,
}

impl<T> Streamstats<T>
where
    T: Value,
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
            assert_eq!(streamstats.percentile(1.0), Ok(i));
        }
    }
}
