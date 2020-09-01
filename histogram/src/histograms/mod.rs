// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::error::HistogramError;
use crate::AtomicCounter;
use crate::Counter;
use crate::Indexing;
use rustcommon_atomics::{Atomic, Ordering};

/// A histogram type which may be concurrently modified across threads because
/// it uses atomic counters. All operations are performed using a `Relaxed`
/// ordering.
pub struct AtomicHistogram<Value, Count> {
    buckets: Vec<Count>,
    exact: Value,
    max: Value,
    precision: u8,
    too_high: Count,
}

impl<Value, Count> AtomicHistogram<Value, Count>
where
    Value: Indexing,
    Count: AtomicCounter + Default,
    u64: From<Value> + From<<Count as Atomic>::Primitive>,
    <Count as Atomic>::Primitive: Copy,
{
    /// Create a new atomic histogram. Stores values from 0 to max. Precision
    /// is used to specify how many significant figures will be preserved.
    pub fn new(max: Value, precision: u8) -> Self {
        let precision = Value::constrain_precision(precision);
        let exact = Value::constrain_exact(max, precision);

        let mut histogram = Self {
            buckets: Vec::new(),
            exact,
            max,
            precision,
            too_high: Count::default(),
        };

        // initialize buckets
        let max_index = Value::get_index(max, max, exact, precision).unwrap();
        let mut buckets = Vec::with_capacity(max_index + 1);
        for _ in 0..=max_index {
            buckets.push(Count::default());
        }
        histogram.buckets = buckets;

        histogram
    }

    /// Increment the value by the provided count, may saturate the bucket's
    /// counter.
    pub fn increment(&self, value: Value, count: <Count as Atomic>::Primitive) {
        if let Ok(index) = Value::get_index(value, self.max, self.exact, self.precision) {
            self.buckets[index].fetch_saturating_add(count, Ordering::Relaxed);
        } else {
            self.too_high.fetch_saturating_add(count, Ordering::Relaxed);
        }
    }

    /// Decrement the value by the provided count, may saturate at zero.
    pub fn decrement(&self, value: Value, count: <Count as Atomic>::Primitive) {
        if let Ok(index) = Value::get_index(value, self.max, self.exact, self.precision) {
            self.buckets[index].fetch_saturating_sub(count, Ordering::Relaxed);
        } else {
            self.too_high.fetch_saturating_sub(count, Ordering::Relaxed);
        }
    }

    /// Clear all counts
    pub fn clear(&self) {
        let default = Count::default().load(Ordering::Relaxed);
        for i in 0..self.buckets.len() {
            self.buckets[i].store(default, Ordering::Relaxed);
        }
        self.too_high.store(default, Ordering::Relaxed);
    }

    /// Return the value closest to the specified percentile. Returns an error
    /// if the value is outside of the histogram range or if the histogram is
    /// empty.
    pub fn percentile(&self, percentile: f64) -> Result<Value, HistogramError> {
        let mut total = 0_u64;
        for value in self.buckets.iter() {
            total += u64::from(value.load(Ordering::Relaxed));
        }
        total += u64::from(self.too_high.load(Ordering::Relaxed));
        if total == 0 {
            return Err(HistogramError::Empty);
        }
        let need = if percentile > 0.0 {
            (percentile * total as f64).ceil() as u64
        } else {
            1
        };
        let mut have = 0_u64;
        for i in 0..self.buckets.len() {
            have += u64::from(self.buckets[i].load(Ordering::Relaxed));
            if have >= need {
                return Ok(Value::get_value(
                    i,
                    self.buckets.len(),
                    self.max,
                    self.exact,
                    self.precision,
                )
                .unwrap());
            }
        }
        Err(HistogramError::OutOfRange)
    }
}

/// A histogram type which follows normal ownership rules.
pub struct Histogram<Value, Count> {
    buckets: Vec<Count>,
    exact: Value,
    max: Value,
    precision: u8,
    too_high: Count,
}

impl<Value, Count> Histogram<Value, Count>
where
    Value: Indexing,
    Count: Counter,
    u64: From<Value> + From<Count>,
{
    /// Create a new histogram. Stores values from 0 to max. Precision is used
    /// to specify how many significant figures will be preserved.
    pub fn new(max: Value, precision: u8) -> Self {
        let precision = Value::constrain_precision(precision);
        let exact = Value::constrain_exact(max, precision);

        let mut histogram = Self {
            buckets: Vec::new(),
            exact,
            max,
            precision,
            too_high: Count::default(),
        };

        // initialize buckets
        let max_index = Value::get_index(max, max, exact, precision).unwrap();
        let mut buckets = Vec::with_capacity(max_index + 1);
        for _ in 0..=max_index {
            buckets.push(Count::default());
        }
        histogram.buckets = buckets;

        histogram
    }

    /// Increment the value by the provided count, may saturate the bucket's
    /// counter.
    pub fn increment(&mut self, value: Value, count: Count) {
        if let Ok(index) = Value::get_index(value, self.max, self.exact, self.precision) {
            self.buckets[index].saturating_add(count);
        } else {
            self.too_high.saturating_add(count);
        }
    }

    /// Decrement the value by the provided count, may saturate at zero.
    pub fn decrement(&mut self, value: Value, count: Count) {
        if let Ok(index) = Value::get_index(value, self.max, self.exact, self.precision) {
            self.buckets[index].saturating_sub(count);
        } else {
            self.too_high.saturating_sub(count);
        }
    }

    /// Clear all counts
    pub fn clear(&mut self) {
        for i in 0..self.buckets.len() {
            self.buckets[i] = Count::default();
        }
        self.too_high = Count::default();
    }

    /// Return the value closest to the specified percentile. Returns an error
    /// if the value is outside of the histogram range or if the histogram is
    /// empty.
    pub fn percentile(&self, percentile: f64) -> Result<Value, HistogramError> {
        let mut total = 0_u64;
        for value in self.buckets.iter() {
            total += u64::from(*value);
        }
        total += u64::from(self.too_high);
        if total == 0 {
            return Err(HistogramError::Empty);
        }
        let need = if percentile > 0.0 {
            (percentile * total as f64).ceil() as u64
        } else {
            1
        };
        let mut have = 0_u64;
        for i in 0..self.buckets.len() {
            have += u64::from(self.buckets[i]);
            if have >= need {
                return Ok(Value::get_value(
                    i,
                    self.buckets.len(),
                    self.max,
                    self.exact,
                    self.precision,
                )
                .unwrap());
            }
        }
        Err(HistogramError::OutOfRange)
    }
}
