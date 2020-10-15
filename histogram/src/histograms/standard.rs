// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::{Bucket, Counter, HistogramError, Indexing};

#[derive(Clone)]
/// A histogram structure which stores counts for a range of values.
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
        buckets.shrink_to_fit();
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

    /// Clear all counts.
    pub fn clear(&mut self) {
        for i in 0..self.buckets.len() {
            self.buckets[i] = Count::default();
        }
        self.too_high = Count::default();
    }

    /// Return the number of buckets stored within the histogram.
    pub fn buckets(&self) -> usize {
        self.buckets.len()
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
            total += u64::from(*value);
        }
        total += u64::from(self.too_high);
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

    /// Internal function to get a bucket by index
    fn get_bucket(&self, index: usize) -> Option<Bucket<Value, Count>> {
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
            let count = self.buckets[index];
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
    pub fn sub_assign(&mut self, other: &Self) {
        if u64::from(self.max) == u64::from(other.max) && self.precision == other.precision {
            // fast path when histograms have same configuration
            for i in 0..self.buckets.len() {
                self.buckets[i].saturating_sub(other.buckets[i]);
            }
            self.too_high.saturating_sub(other.too_high);
        } else {
            // slow path if we need to calculate appropriate index for each bucket
            for bucket in other {
                self.decrement(bucket.value, bucket.count);
            }
            self.too_high.saturating_sub(other.too_high);
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
    pub fn add_assign(&mut self, other: &Self) {
        if u64::from(self.max) == u64::from(other.max) && self.precision == other.precision {
            // fast path when histograms have same configuration
            for i in 0..self.buckets.len() {
                self.buckets[i].saturating_add(other.buckets[i]);
            }
            self.too_high.saturating_add(other.too_high);
        } else {
            // slow path if we need to calculate appropriate index for each bucket
            for bucket in other {
                self.increment(bucket.value, bucket.count);
            }
            self.too_high.saturating_add(other.too_high);
        }
    }
}

pub struct Iter<'a, Value, Count>
where
    Value: Indexing,
    Count: Counter,
    u64: From<Value> + From<Count>,
{
    inner: &'a Histogram<Value, Count>,
    index: usize,
}

impl<'a, Value, Count> Iter<'a, Value, Count>
where
    Value: Indexing,
    Count: Counter,
    u64: From<Value> + From<Count>,
{
    fn new(inner: &'a Histogram<Value, Count>) -> Iter<'a, Value, Count> {
        Iter { inner, index: 0 }
    }
}

impl<'a, Value, Count> Iterator for Iter<'a, Value, Count>
where
    Value: Indexing,
    Count: Counter,
    u64: From<Value> + From<Count>,
{
    type Item = Bucket<Value, Count>;

    fn next(&mut self) -> Option<Bucket<Value, Count>> {
        let bucket = self.inner.get_bucket(self.index);
        self.index += 1;
        bucket
    }
}

impl<'a, Value, Count> IntoIterator for &'a Histogram<Value, Count>
where
    Value: Indexing,
    Count: Counter,
    u64: From<Value> + From<Count>,
{
    type Item = Bucket<Value, Count>;
    type IntoIter = Iter<'a, Value, Count>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}
