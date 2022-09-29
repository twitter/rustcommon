// Copyright 2022 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering;

/// A `Histogram` groups recorded values into buckets of similar values and
/// tracks counts for recorded values that fall into those ranges.
#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct Histogram {
    // minimum resolution parameter `M = 2^m`
    m: u32,
    // minimum resolution range parameter `R = 2^r - 1`
    r: u32,
    // maximum value parameter `N = 2^n - 1`
    n: u32,

    // minimum resolution value
    M: u64,
    // minimum resolution upper bound
    R: u64,
    // maximum value
    N: u64,
    // grouping factor
    G: u64,

    // buckets of ranges that hold actual counts
    buckets: Box<[AtomicU32]>,
}

/// A `Builder` allows for constructing a `Histogram` with the desired
/// configuration.
pub struct Builder {
    // minimum resolution parameter `M = 2^m`
    m: u32,
    // minimum resolution range parameter `R = 2^r - 1`
    r: u32,
    // maximum value parameter `N = 2^n - 1`
    n: u32,
}

impl Builder {
    /// Consume the `Builder` and return a `Histogram`.
    pub fn build(self) -> Result<Histogram, Error> {
        Histogram::new(self.m, self.r, self.n)
    }

    /// Sets the width of the smallest bucket in the `Histogram`.
    ///
    /// As the `Histogram` uses base-2 internally, the resolution will be the
    /// largest power of two that is less than or equal to the provided value.
    /// For example, if the minimum resolution is set to 10, the width of the
    /// smallest bucket will be 8.
    pub fn min_resolution(mut self, width: u64) -> Self {
        self.m = 64 - width.leading_zeros();
        self
    }

    /// Sets the maximum value that the minimum resolution extends to.
    ///
    /// This value should be greater than the minimum resolution. If the value
    /// provided is not a power of two, the smallest power of two that is larger
    /// than the provided value will be used.
    pub fn min_resolution_range(mut self, value: u64) -> Self {
        self.r = 64 - value.next_power_of_two().leading_zeros();
        self
    }

    /// Sets the maximum value that can be recorded into the `Histogram`.
    ///
    /// If the value provided is not a power of two, the smallest power of two
    /// that is larger than the provided value will be used.
    pub fn maximum_value(mut self, value: u64) -> Self {
        self.n = 64 - value.next_power_of_two().leading_zeros();
        self
    }
}

impl Histogram {
    /// Construct a new histogram by providing the configuration directly.
    ///
    /// - `m` - sets the minimum resolution `M = 2^m`. This is the smallest unit
    /// of quantification, which is also the smallest bucket width. If the input
    /// values are always integers, choosing `m=0` would ensure precise
    /// recording for the smallest values.
    ///
    /// - `r` - sets the minimum resolution range `R = 2^r - 1`. The selected
    /// value must be greater than the minimum resolution `m`. This sets the
    /// maximum value that the minimum resolution should extend to.
    ///
    /// - `n` - sets the maximum value `N = 2^n - 1`. The selected value must be
    /// greater than or equal to the minimum resolution range `r`.
    ///
    /// # Panics
    /// This will panic if an invalid configuration is specified.
    #[allow(non_snake_case)]
    pub fn new(m: u32, r: u32, n: u32) -> Result<Self, Error> {
        if r <= m || r > n || n > 64 {
            return Err(Error::InvalidConfig);
        }

        let M = 1 << m;
        let R = (1 << r) - 1;
        let N = (1 << n) - 1;
        let G: u64 = 1 << (r - m - 1);

        let n_buckets = (n - r + 2) as u64 * G;

        let mut buckets = Vec::new();
        buckets.resize_with(n_buckets as usize, || AtomicU32::new(0));

        Ok(Self {
            m,
            r,
            n,
            M,
            R,
            N,
            G,
            buckets: buckets.into_boxed_slice(),
        })
    }

    /// Creates a `Builder` with the default values `m = 0`, `r = 10`, `n = 30`.
    ///
    /// This would create a `Histogram` with 11264 buckets which can store
    /// values from 1 to 1_073_741_823 with values 1 to 1023 being stored in
    /// buckets with a width of 1. Such a `Histogram` would be appropriate for
    /// latencies measured in nanoseconds where the max expected latency is one
    /// second.
    pub fn builder() -> Builder {
        Builder { m: 0, r: 10, n: 30 }
    }

    /// Resets the `Histogram` by zeroing out the count for every bucket.
    pub fn clear(&self) {
        for bucket in self.buckets.iter() {
            bucket.store(0, Ordering::Relaxed);
        }
    }

    /// Increment the histogram bucket corresponding to the provided `value` by
    /// the provided `count`.
    ///
    /// This operation wraps on overflow.
    #[allow(clippy::result_unit_err)]
    pub fn increment(&self, value: u64, count: u32) -> Result<(), Error> {
        if value > self.N {
            // value too big
            return Err(Error::OutOfRange);
        }

        let index = self.bucket_index(value);
        self.buckets[index].fetch_add(count, Ordering::Relaxed);

        Ok(())
    }

    /// Decrement the histogram bucket corresponding to the provided `value` by
    /// the provided `count`.
    ///
    /// This operation wraps on overflow.
    #[allow(clippy::result_unit_err)]
    pub fn decrement(&self, value: u64, count: u32) -> Result<(), Error> {
        if value > self.N {
            // value too big
            return Err(Error::OutOfRange);
        }

        let index = self.bucket_index(value);
        self.buckets[index].fetch_add(count, Ordering::Relaxed);

        Ok(())
    }

    /// Retrieve the `Bucket` which corresponds to the provided percentile.
    ///
    /// An error will be returned if the percentile is invalid or if there are
    /// no samples in the `Histogram`.
    ///
    /// Note: if you are reporting on multiple percentiles, it is more efficient
    /// to use the `percentiles` function to retrieve multiple percentiles in a
    /// single call.
    pub fn percentile(&self, percentile: f64) -> Result<Bucket, Error> {
        if !(0.0..=100.0).contains(&percentile) {
            return Err(Error::InvalidPercentile);
        }

        let total: u64 = self
            .buckets
            .iter()
            .map(|v| v.load(Ordering::Relaxed) as u64)
            .sum();
        if total == 0 {
            return Err(Error::Empty);
        }

        let mut threshold = (percentile * total as f64 / 100.0).ceil() as u64;
        if threshold == 0 {
            threshold += 1;
        }

        let mut seen = 0;
        let mut max = 0;

        for (id, count) in self
            .buckets
            .iter()
            .map(|b| b.load(Ordering::Relaxed) as u64)
            .enumerate()
        {
            if count > 0 {
                max = id;
            }

            seen += count;

            if seen >= threshold {
                return Ok(self.get_bucket(id));
            }
        }

        // if a bucket can't be found for the percentile, return the max bucket
        // seen while walking the histogram. this may be necessary if there is a
        // concurrent modification that reduces the counts before we have a
        // chance to get to that bucket
        Ok(self.get_bucket(max))
    }

    /// Returns a set of percentiles in a single and efficient bulk operation.
    /// Note that the returned percentiles will be sorted from lowest to highest
    /// in the result, even if they do not appear in that order in the provided
    /// set of requested percentiles.
    pub fn percentiles(&self, percentiles: &[f64]) -> Result<Vec<Percentile>, Error> {
        let mut percentiles = percentiles.to_owned();
        percentiles.sort_by(|a, b| a.partial_cmp(b).unwrap());

        for percentile in &percentiles {
            if !(0.0..=100.0).contains(percentile) {
                return Err(Error::InvalidPercentile);
            }
        }

        let total: u64 = self
            .buckets
            .iter()
            .map(|v| v.load(Ordering::Relaxed) as u64)
            .sum();
        if total == 0 {
            return Err(Error::Empty);
        }

        let thresholds: Vec<u64> = percentiles
            .iter()
            .map(|v| std::cmp::min(1, (v * total as f64 / 100.0).ceil() as u64))
            .collect();

        let mut max = 0;
        let mut seen = 0;
        let mut threshold_idx = 0;

        let mut result = Vec::with_capacity(thresholds.len());

        for (bucket_idx, count) in self
            .buckets
            .iter()
            .map(|b| b.load(Ordering::Relaxed) as u64)
            .enumerate()
        {
            if count > 0 {
                max = bucket_idx;
            }

            seen += count;
            while seen > thresholds[threshold_idx] && threshold_idx < thresholds.len() {
                result.push(Percentile {
                    percentile: percentiles[threshold_idx],
                    bucket: self.get_bucket(bucket_idx),
                });

                threshold_idx += 1;
            }

            if threshold_idx >= thresholds.len() {
                break;
            }
        }

        // pad the results with the max bucket seen while walking the histogram
        // this may be necessary if there is a concurrent modification that
        // reduces the counts before we have a chance to get to that bucket
        while result.len() < percentiles.len() {
            // get the index within the percentiles vec
            let idx = percentiles.len() - result.len() - 1;
            result.push(Percentile {
                percentile: percentiles[idx],
                bucket: self.get_bucket(max),
            });
        }

        Ok(result)
    }

    /// Adds the other `Histogram` to this `Histogram`. Returns an error if
    /// there are differences in the configurations of both `Histogram`s.
    #[allow(clippy::result_unit_err)]
    pub fn add_assign(&self, other: &Self) -> Result<(), Error> {
        // make sure they match
        if self.m != other.m || self.r != other.r || self.n != other.n {
            return Err(Error::IncompatibleHistogram);
        }

        for (idx, value) in other
            .buckets
            .iter()
            .map(|v| v.load(Ordering::Relaxed))
            .enumerate()
        {
            self.buckets[idx].fetch_add(value, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Subtracts the other `Histogram` from this `Histogram`. Returns an error
    /// if there are differences in the configurations of both `Histogram`s.
    pub fn sub_assign(&self, other: &Self) -> Result<(), Error> {
        // make sure they match
        if self.m != other.m || self.r != other.r || self.n != other.n {
            return Err(Error::IncompatibleHistogram);
        }

        for (idx, value) in other
            .buckets
            .iter()
            .map(|v| v.load(Ordering::Relaxed))
            .enumerate()
        {
            self.buckets[idx].fetch_sub(value, Ordering::Relaxed);
        }

        Ok(())
    }

    pub fn buckets(&self) -> usize {
        self.buckets.len()
    }

    fn low(&self, idx: usize) -> u64 {
        let idx = idx as u64;
        let m = self.m as u64;
        let r = self.r as u64;
        let g = idx >> (self.r - self.m - 1);
        let b = idx - g * self.G;

        if g < 1 {
            (1 << m) * b
        } else {
            (1 << (r + g - 2)) + (1 << (m + g - 1)) * b
        }
    }

    fn high(&self, idx: usize) -> u64 {
        let idx = idx as u64;
        let m = self.m as u64;
        let r = self.r as u64;
        let g = idx >> (self.r - self.m - 1);
        let b = idx - g * self.G + 1;

        if g < 1 {
            (1 << m) * b - 1
        } else {
            (1 << (r + g - 2)) + (1 << (m + g - 1)) * b - 1
        }
    }

    fn get_bucket(&self, idx: usize) -> Bucket {
        let low = self.low(idx);
        let high = self.high(idx);

        Bucket {
            low,
            high,
            count: self.buckets[idx].load(Ordering::Relaxed),
        }
    }

    fn bucket_index(&self, value: u64) -> usize {
        if value == 0 {
            return 0;
        }

        let m = self.m as u64;
        let r = self.r as u64;

        let h = (63 - value.leading_zeros()) as u64;

        if h < r {
            (value >> m) as usize
        } else {
            let d = h - r + 1;
            ((d + 1) * self.G + ((value - (1 << h)) >> (m + d))) as usize
        }
    }
}

impl Clone for Histogram {
    fn clone(&self) -> Self {
        // SAFETY: unwrap is safe because we already have a histogram with these
        // values for the parameters
        let ret = Histogram::new(self.m as u32, self.r as u32, self.n as u32).unwrap();
        for (id, value) in self
            .buckets
            .iter()
            .map(|v| v.load(Ordering::Relaxed))
            .enumerate()
        {
            ret.buckets[id].store(value, Ordering::Relaxed)
        }
        ret
    }
}

/// An iterator that allows walking through the `Bucket`s within a `Histogram`.
pub struct HistogramIter<'a> {
    current: usize,
    histogram: &'a Histogram,
}

impl<'a> IntoIterator for &'a Histogram {
    type Item = Bucket;
    type IntoIter = HistogramIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        HistogramIter {
            current: 0,
            histogram: self,
        }
    }
}

impl<'a> Iterator for HistogramIter<'a> {
    type Item = Bucket;

    fn next(&mut self) -> Option<Bucket> {
        if self.current < self.histogram.buckets.len() {
            let bucket = self.histogram.get_bucket(self.current);
            self.current += 1;
            Some(bucket)
        } else {
            None
        }
    }
}
