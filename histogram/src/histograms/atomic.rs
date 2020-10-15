// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Counter;
use crate::Histogram;
use crate::{AtomicCounter, Bucket, HistogramError, Indexing};
use rustcommon_atomics::{Atomic, Ordering};

/// A histogram type which may be concurrently modified across threads because
/// it uses atomic counters. All operations are performed using a relaxed
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
        buckets.shrink_to_fit();
        histogram.buckets = buckets;

        histogram
    }

    /// Return the number of buckets stored within this histogram.
    pub fn buckets(&self) -> usize {
        self.buckets.len()
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

    /// Clear all counts.
    pub fn clear(&self) {
        let default = Count::default().load(Ordering::Relaxed);
        for i in 0..self.buckets.len() {
            self.buckets[i].store(default, Ordering::Relaxed);
        }
        self.too_high.store(default, Ordering::Relaxed);
    }

    /// Return the value closest to the specified percentile. Returns an error
    /// if the value is outside of the histogram range or if the histogram is
    /// empty. Percentile must be within the range 0.0 to 100.0
    pub fn percentile(&self, percentile: f64) -> Result<Value, HistogramError> {
        if percentile < 0.0 || percentile > 100.0 {
            return Err(HistogramError::InvalidPercentile);
        }
        let mut total = 0_u64;
        for value in self.buckets.iter() {
            total += u64::from(value.load(Ordering::Relaxed));
        }
        total += u64::from(self.too_high.load(Ordering::Relaxed));
        if total == 0 {
            return Err(HistogramError::Empty);
        }
        let need = if percentile > 0.0 {
            (percentile / 100.0 * total as f64).ceil() as u64
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

    /// Internal function to get the bucket at a given index
    fn get_bucket(&self, index: usize) -> Option<Bucket<Value, <Count as Atomic>::Primitive>> {
        if let Ok(min) = Value::get_min_value(
            index,
            self.buckets.len(),
            self.max,
            self.exact,
            self.precision,
        ) {
            let value = Value::get_value(
                index,
                self.buckets.len(),
                self.max,
                self.exact,
                self.precision,
            )
            .unwrap();
            let max = Value::get_max_value(
                index,
                self.buckets.len(),
                self.max,
                self.exact,
                self.precision,
            )
            .unwrap();
            let count = self.buckets[index].load(Ordering::Relaxed);
            Some(Bucket {
                min,
                max,
                value,
                count,
            })
        } else {
            None
        }
    }

    /// Subtracts another histogram from this histogram
    ///
    /// NOTES:
    /// If the histograms differ in their configured range, we treat the samples
    /// that were too high on the right hand side as if they would also be too
    /// high on the histogram those counts are subtracted from. This may produce
    /// unexpected results if subtracting a histogram with a smaller range from
    /// one with a wider range.
    ///
    /// If the histograms differ in their configured precision, unusual
    /// artifacts may be introduced by subtracting a low precision histogram
    /// from one with higher precision.
    pub fn sub_assign(&self, other: &Self) {
        if u64::from(self.max) == u64::from(other.max) && self.precision == other.precision {
            // fast path when histograms have same configuration
            for i in 0..self.buckets.len() {
                self.buckets[i].fetch_saturating_sub(
                    other.buckets[i].load(Ordering::Relaxed),
                    Ordering::Relaxed,
                );
                self.too_high.fetch_saturating_sub(
                    other.too_high.load(Ordering::Relaxed),
                    Ordering::Relaxed,
                );
            }
        } else {
            // slow path if we need to calculate appropriate index for each bucket
            for bucket in other {
                self.decrement(bucket.value, bucket.count);
            }
            self.too_high
                .fetch_saturating_sub(other.too_high.load(Ordering::Relaxed), Ordering::Relaxed);
        }
    }

    /// Adds another histogram to this histogram
    ///
    /// NOTES:
    /// If the histograms differ in their configured range, we treat the samples
    /// that were too high on the right hand side as if they would also be too
    /// high on the histogram those counts are added to. This may produce
    /// unexpected results if adding a histogram with a smaller range to one
    /// with a wider range.
    ///
    /// If the histograms differ in their configured precision, unusual
    /// artifacts may be introduced by adding a low precision histogram to one
    /// with higher precision.
    pub fn add_assign(&self, other: &Self) {
        if u64::from(self.max) == u64::from(other.max) && self.precision == other.precision {
            // fast path when histograms have same configuration
            for i in 0..self.buckets.len() {
                self.buckets[i].fetch_saturating_add(
                    other.buckets[i].load(Ordering::Relaxed),
                    Ordering::Relaxed,
                );
                self.too_high.fetch_saturating_add(
                    other.too_high.load(Ordering::Relaxed),
                    Ordering::Relaxed,
                );
            }
        } else {
            // slow path if we need to calculate appropriate index for each bucket
            for bucket in other {
                self.increment(bucket.value, bucket.count);
            }
            self.too_high
                .fetch_saturating_add(other.too_high.load(Ordering::Relaxed), Ordering::Relaxed);
        }
    }

    /// Convert this `AtomicHistogram` to a non-atomic version by allocating a
    /// new histogram and performing relaxed loads.
    ///
    /// Note: users needing stronger consistency should ensure that no other
    /// threads are writing to the histogram while this operation is
    /// in-progress.
    pub fn load(&self) -> Histogram<Value, <Count as Atomic>::Primitive>
    where
        Value: Copy + std::ops::Sub<Output = Value>,
        <Count as Atomic>::Primitive: Counter,
    {
        let mut result = Histogram::new(self.max, self.precision);
        for bucket in self {
            result.increment(bucket.value(), bucket.count());
        }
        result
    }
}

pub struct Iter<'a, Value, Count>
where
    Value: Indexing,
    Count: AtomicCounter + Default,
    u64: From<Value> + From<<Count as Atomic>::Primitive>,
    <Count as Atomic>::Primitive: Copy,
{
    inner: &'a AtomicHistogram<Value, Count>,
    index: usize,
}

impl<'a, Value, Count> Iter<'a, Value, Count>
where
    Value: Indexing,
    Count: AtomicCounter + Default,
    u64: From<Value> + From<<Count as Atomic>::Primitive>,
    <Count as Atomic>::Primitive: Copy,
{
    fn new(inner: &'a AtomicHistogram<Value, Count>) -> Iter<'a, Value, Count> {
        Iter { inner, index: 0 }
    }
}

impl<'a, Value, Count> Iterator for Iter<'a, Value, Count>
where
    Value: Indexing,
    Count: AtomicCounter + Default,
    u64: From<Value> + From<<Count as Atomic>::Primitive>,
    <Count as Atomic>::Primitive: Copy,
{
    type Item = Bucket<Value, <Count as Atomic>::Primitive>;

    fn next(&mut self) -> Option<Bucket<Value, <Count as Atomic>::Primitive>> {
        let bucket = self.inner.get_bucket(self.index);
        self.index += 1;
        bucket
    }
}

impl<'a, Value, Count> IntoIterator for &'a AtomicHistogram<Value, Count>
where
    Value: Indexing,
    Count: AtomicCounter + Default,
    u64: From<Value> + From<<Count as Atomic>::Primitive>,
    <Count as Atomic>::Primitive: Copy,
{
    type Item = Bucket<Value, <Count as Atomic>::Primitive>;
    type IntoIter = Iter<'a, Value, Count>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}
