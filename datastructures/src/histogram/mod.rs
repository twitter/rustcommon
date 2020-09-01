// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

use thiserror::Error;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

type SharedVecDeque<T> = Arc<Mutex<VecDeque<T>>>;

/// `Histogram` is inspired by HDRHistogram and stores a counter for `Bucket`s
/// across a range of input values. `Histogram`s store between 0 and the `max`
/// value passed to the constructor. Optionally, a `Histogram` may retain
/// samples across a given `window`, resulting in a moving `Histogram` which
/// holds only recently seen samples. Additionally, the number of samples
/// retained can be bounded by the `capacity` of the `Histogram`. `Histogram` is
/// generic across `AtomicPrimitive`s which implement `Saturating`. This allows
/// the user to customize the `Histogram` to trade-off between being able to
/// hold larger counts per `Bucket` or use less memory by selecting which type
/// to use for the counters.
pub struct Histogram<T>
where
    T: Atomic + SaturatingArithmetic + Unsigned + Default,
    <T as Atomic>::Primitive: Default + PartialEq + Copy,
{
    exact: AtomicU64,
    max: AtomicU64,
    buckets: Vec<T>,
    index: Vec<AtomicU64>,
    too_high: AtomicU64,
    precision: AtomicU32,
    samples: Option<SharedVecDeque<Sample<<T as Atomic>::Primitive>>>,
    window: Option<Arc<Mutex<Duration>>>,
    capacity: Option<AtomicUsize>,
}

#[derive(Error, Debug, PartialEq)]
pub enum HistogramError {
    #[error("empty histogram")]
    Empty,
}

/// Indicates whether the sample was an `Increment` or a `Decrement` operation
enum Direction {
    Decrement,
    Increment,
}

/// Used to hold samples in the internal buffer, later used to remove expired
/// or excess samples
struct Sample<T> {
    value: u64,
    count: T,
    time: Instant,
    direction: Direction,
}

/// A `Bucket` stores a count across a range of values
pub struct Bucket<T> {
    min: u64,
    max: u64,
    value: u64,
    count: T,
}

impl<T> Bucket<T>
where
    T: Copy,
{
    /// Return the inclusive minimum of all values storable in the `Bucket`
    pub fn min(&self) -> u64 {
        self.min
    }

    /// Return the non-inclusive maximum of all values storable in the `Bucket`
    pub fn max(&self) -> u64 {
        self.max
    }

    /// Return the nominal value of this `Bucket`
    pub fn value(&self) -> u64 {
        self.value
    }

    /// Return the count of values recorded into this `Bucket`
    pub fn count(&self) -> T {
        self.count
    }

    /// Returns the range of values storable in this `Bucket`
    pub fn width(&self) -> u64 {
        self.max - self.min
    }
}

impl<T> Histogram<T>
where
    T: Atomic + Unsigned + SaturatingArithmetic + Default,
    <T as Atomic>::Primitive: Default + PartialEq + Copy,
    u64: std::convert::From<<T as Atomic>::Primitive>,
{
    /// Create a new `Histogram` which stores from 0..`max`, with at least
    /// `precision` digits represented exactly. Optionally, specify a `window`
    /// beyond which, aged sampled will be removed from the `Histogram`.
    /// Optionally, bound the number of samples stored in the `Histogram` by
    /// specifying the `capacity`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_datastructures::*;
    ///
    /// // this creates a histogram that will hold values from 0..100 with all
    /// // values represented exactly using 8-bit counters for each bucket
    /// let x = Histogram::<AtomicU8>::new(100, 2, None, None);
    ///
    /// // this creates a histogram that will hold values from 0 nanoseconds to
    /// // 1 second, stored as nanoseconds with the most significant 3 digits
    /// // represented exactly using 64-bit counters for each bucket
    /// let x = Histogram::<AtomicU64>::new(60_000_000_000, 3, None, None);
    ///
    /// // this creates a histogram that contains only samples from the past
    /// // minute using 32-bit counters for each bucket
    /// use std::time::Duration;
    /// let x = Histogram::<AtomicU32>::new(100, 2, Some(Duration::new(60, 0)), None);
    ///
    /// // this creates a histogram that contains only the past 100 samples
    /// // using 32-bit counters for each bucket
    /// let x = Histogram::<AtomicU32>::new(100, 2, None, Some(100));
    ///
    /// // this creates a histogram that contains at most 100 samples from the
    /// // past minute using 32-bit counters for each bucket
    /// let x = Histogram::<AtomicU32>::new(100, 2, Some(Duration::new(60, 0)), Some(100));
    /// ```
    pub fn new(
        max: u64,
        precision: u32,
        window: Option<Duration>,
        capacity: Option<usize>,
    ) -> Self {
        let mut histogram: Histogram<T> = Histogram {
            exact: AtomicU64::new(10_u64.pow(precision)),
            max: AtomicU64::new(max),
            buckets: Vec::new(),
            index: Vec::new(),
            too_high: AtomicU64::new(0),
            precision: AtomicU32::new(precision),
            samples: None,
            window: None,
            capacity: None,
        };
        if let Some(window) = window {
            histogram.window = Some(Arc::new(Mutex::new(window)));
            if let Some(capacity) = capacity {
                histogram.samples = Some(Arc::new(Mutex::new(VecDeque::with_capacity(capacity))));
                histogram.capacity = Some(AtomicUsize::new(capacity));
            } else {
                histogram.samples = Some(Arc::new(Mutex::new(VecDeque::with_capacity(1))));
            }
        } else if let Some(capacity) = capacity {
            histogram.samples = Some(Arc::new(Mutex::new(VecDeque::with_capacity(capacity))));
            histogram.capacity = Some(AtomicUsize::new(capacity));
        }
        for _ in 0..=histogram.get_index(max).unwrap() {
            histogram.buckets.push(T::default());
        }
        histogram.buckets.shrink_to_fit();
        for _ in 0..=(histogram.get_index(max).unwrap() / 100) {
            histogram.index.push(AtomicU64::default());
        }
        histogram.index.shrink_to_fit();
        histogram
    }

    /// Returns the total size of the `Histogram` in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_datastructures::*;
    ///
    /// let x = Histogram::<AtomicU64>::new(1_000_000, 3, None, None);
    /// assert_eq!(x.size(), 30024); // the histogram is ~30KB
    ///
    /// let x = Histogram::<AtomicU32>::new(1_000_000, 3, None, None);
    /// assert_eq!(x.size(), 15220); // using smaller counters makes it ~15KB
    ///
    /// let x = Histogram::<AtomicU64>::new(1_000_000, 2, None, None);
    /// assert_eq!(x.size(), 3840); // reducing the precision makes it ~4KB
    /// ```
    pub fn size(&self) -> usize {
        let mut total_size = 0;
        // add the struct overhead
        total_size += std::mem::size_of::<Histogram<T>>();
        // add the bucket storage
        total_size += std::mem::size_of::<T>() * self.buckets.capacity();
        // add the index storage
        total_size += std::mem::size_of::<AtomicU64>() * self.index.capacity();
        // add the samples storage
        if let Some(samples) = &self.samples {
            let samples = samples.lock().unwrap();
            total_size += std::mem::size_of::<Sample<T>>() * samples.capacity();
        }
        total_size
    }

    // Internal function to get the index for a given value
    fn get_index(&self, value: u64) -> Result<usize, ()> {
        if value > self.max.load(Ordering::Relaxed) {
            Err(())
        } else if value <= self.exact.load(Ordering::Relaxed) {
            Ok(value as usize)
        } else {
            let exact = self.exact.load(Ordering::Relaxed) as usize;
            let power = (value as f64).log10().floor() as u32;
            let divisor = 10_u64.pow((power - self.precision.load(Ordering::Relaxed)) as u32 + 1);
            let power_offset = (0.9
                * f64::from(exact as u32 * (power - self.precision.load(Ordering::Relaxed))))
                as usize;
            let remainder = value / divisor as u64;
            let shift = exact / 10;
            let index = exact + power_offset + remainder as usize - shift;
            Ok(index)
        }
    }

    // Internal function to get the minimum value for a given bucket index
    fn get_min_value(&self, index: usize) -> Result<u64, ()> {
        if index >= self.buckets.len() {
            Err(())
        } else if (index as u64) <= self.exact.load(Ordering::Relaxed) {
            Ok(index as u64)
        } else if index == self.buckets.len() - 1 {
            Ok(self.max.load(Ordering::Relaxed))
        } else {
            let shift = 10_usize.pow(self.precision.load(Ordering::Relaxed) - 1);
            let base_offset = 10_usize.pow(self.precision.load(Ordering::Relaxed));
            let power = self.precision.load(Ordering::Relaxed) as usize
                + (index - base_offset)
                    / (9 * 10_usize.pow(self.precision.load(Ordering::Relaxed) - 1));
            let power_offset = (0.9
                * (10_usize.pow(self.precision.load(Ordering::Relaxed))
                    * (power - self.precision.load(Ordering::Relaxed) as usize))
                    as f64) as usize;
            let value = (index + shift - base_offset - power_offset) as u64
                * 10_u64.pow((power - self.precision.load(Ordering::Relaxed) as usize + 1) as u32);
            Ok(value)
        }
    }

    // Internal function to get the non-inclusive maximum for a bucket at index
    fn get_max_value(&self, index: usize) -> Result<u64, ()> {
        if index == self.buckets.len() - 1 {
            Ok(self.max.load(Ordering::Relaxed) + 1)
        } else {
            Ok(self.get_min_value(index + 1).unwrap())
        }
    }

    // Internal function to get the bucket at a given index
    fn get_bucket(&self, index: usize) -> Option<Bucket<<T as Atomic>::Primitive>> {
        if let Some(counter) = self.buckets.get(index) {
            Some(Bucket {
                min: self.get_min_value(index).unwrap(),
                max: self.get_max_value(index).unwrap(),
                value: self.get_value(index).unwrap(),
                count: counter.load(Ordering::Relaxed),
            })
        } else {
            None
        }
    }

    // Internal function to get the nominal value of the bucket at index
    fn get_value(&self, index: usize) -> Result<u64, ()> {
        self.get_max_value(index).map(|v| v - 1)
    }

    /// Increment the `Bucket` holding `value` by `count`
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_datastructures::*;
    ///
    /// let x = Histogram::<AtomicU64>::new(100, 2, None, None);
    /// x.increment(42, 1);
    /// assert_eq!(x.total_count(), 1);
    /// ```
    pub fn increment(&self, value: u64, count: <T as Atomic>::Primitive) {
        match self.get_index(value) {
            Ok(index) => {
                self.buckets[index].fetch_saturating_add(count, Ordering::Relaxed);
                self.index[index / 100].fetch_saturating_add(u64::from(count), Ordering::Relaxed);
            }
            Err(_) => {
                self.too_high
                    .fetch_saturating_add(u64::from(count), Ordering::Relaxed);
            }
        }
        if let Some(samples) = &self.samples {
            let time = Instant::now();
            self.trim(time);
            let mut samples = samples.lock().unwrap();
            samples.push_back(Sample {
                value,
                count,
                time,
                direction: Direction::Increment,
            });
        }
    }

    /// Decrement the `Bucket` holding `value` by `count`
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_datastructures::*;
    ///
    /// let x = Histogram::<AtomicU64>::new(100, 2, None, None);
    /// x.decrement(42, 1);
    /// assert_eq!(x.total_count(), 0);
    /// ```
    pub fn decrement(&self, value: u64, count: <T as Atomic>::Primitive) {
        match self.get_index(value) {
            Ok(index) => {
                self.buckets[index].fetch_saturating_sub(count, Ordering::Relaxed);
                self.index[index / 100].fetch_saturating_sub(u64::from(count), Ordering::Relaxed);
            }
            Err(_) => {
                self.too_high
                    .fetch_saturating_sub(u64::from(count), Ordering::Relaxed);
            }
        }
        if let Some(samples) = &self.samples {
            let time = Instant::now();
            self.trim(time);
            let mut samples = samples.lock().unwrap();
            samples.push_back(Sample {
                value,
                count,
                time,
                direction: Direction::Decrement,
            });
        }
    }

    /// Clears all `Bucket`s within the `Histogram`
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_datastructures::*;
    ///
    /// let x = Histogram::<AtomicU64>::new(100, 2, None, None);
    /// x.increment(42, 1);
    /// assert_eq!(x.total_count(), 1);
    /// x.clear();
    /// assert_eq!(x.total_count(), 0);
    /// ```
    pub fn clear(&self) {
        if let Some(samples) = &self.samples {
            let mut samples = samples.lock().unwrap();
            samples.clear();
        }
        for i in 0..self.buckets.len() {
            self.buckets[i].store(<T as Atomic>::Primitive::default(), Ordering::Relaxed);
        }
        for i in 0..self.index.len() {
            self.index[i].store(0, Ordering::Relaxed);
        }
        self.too_high.store(0, Ordering::Relaxed);
    }

    // Internal function to remove expired and/or excess samples
    fn trim(&self, time: Instant) {
        if let Some(samples) = &self.samples {
            if let Some(window) = &self.window {
                let window = *window.lock().unwrap();
                let mut samples = samples.lock().unwrap();
                while let Some(sample) = samples.pop_front() {
                    let age = time - sample.time;
                    if age > window {
                        match self.get_index(sample.value) {
                            Ok(index) => match sample.direction {
                                Direction::Decrement => {
                                    self.buckets[index]
                                        .fetch_saturating_add(sample.count, Ordering::Relaxed);
                                    self.index[index / 100].fetch_saturating_add(
                                        u64::from(sample.count),
                                        Ordering::Relaxed,
                                    );
                                }
                                Direction::Increment => {
                                    self.buckets[index]
                                        .fetch_saturating_sub(sample.count, Ordering::Relaxed);
                                    self.index[index / 100].fetch_saturating_sub(
                                        u64::from(sample.count),
                                        Ordering::Relaxed,
                                    );
                                }
                            },
                            Err(_) => match sample.direction {
                                Direction::Decrement => {
                                    self.too_high.fetch_saturating_add(
                                        u64::from(sample.count),
                                        Ordering::Relaxed,
                                    );
                                }
                                Direction::Increment => {
                                    self.too_high.fetch_saturating_sub(
                                        u64::from(sample.count),
                                        Ordering::Relaxed,
                                    );
                                }
                            },
                        }
                    } else {
                        samples.push_front(sample);
                        break;
                    }
                }
                samples.shrink_to_fit();
            }
            if let Some(capacity) = &self.capacity {
                let capacity = capacity.load(Ordering::Relaxed);
                let mut samples = samples.lock().unwrap();
                while samples.len() > capacity {
                    if let Some(sample) = samples.pop_front() {
                        match self.get_index(sample.value) {
                            Ok(index) => match sample.direction {
                                Direction::Decrement => {
                                    self.buckets[index]
                                        .fetch_saturating_add(sample.count, Ordering::Relaxed);
                                    self.index[index / 100].fetch_saturating_add(
                                        u64::from(sample.count),
                                        Ordering::Relaxed,
                                    );
                                }
                                Direction::Increment => {
                                    self.buckets[index]
                                        .fetch_saturating_sub(sample.count, Ordering::Relaxed);
                                    self.index[index / 100].fetch_saturating_sub(
                                        u64::from(sample.count),
                                        Ordering::Relaxed,
                                    );
                                }
                            },
                            Err(_) => match sample.direction {
                                Direction::Decrement => {
                                    self.too_high.fetch_saturating_add(
                                        u64::from(sample.count),
                                        Ordering::Relaxed,
                                    );
                                }
                                Direction::Increment => {
                                    self.too_high.fetch_saturating_sub(
                                        u64::from(sample.count),
                                        Ordering::Relaxed,
                                    );
                                }
                            },
                        }
                    }
                }
                samples.shrink_to_fit();
            }
        }
    }

    /// Returns the total count for all values in the `Histogram`
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_datastructures::*;
    ///
    /// let x = Histogram::<AtomicU64>::new(100, 2, None, None);
    /// x.increment(42, 1);
    /// assert_eq!(x.total_count(), 1);
    /// ```
    pub fn total_count(&self) -> u64 {
        if self.samples.is_some() {
            let time = Instant::now();
            self.trim(time);
        }
        let mut total = 0;
        for i in 0..self.index.len() {
            total += self.index[i].load(Ordering::Relaxed);
        }
        total += self.too_high.load(Ordering::Relaxed);
        total
    }

    /// Returns the nominal value at the percentile specified from 0.0-1.0
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_datastructures::*;
    ///
    /// let x = Histogram::<AtomicU64>::new(100, 2, None, None);
    /// x.increment(42, 1);
    /// assert_eq!(x.percentile(0.0), Ok(42));
    /// assert_eq!(x.percentile(1.0), Ok(42));
    ///
    /// let y = Histogram::<AtomicU64>::new(100, 2, None, None);
    /// for v in 0..100 {
    ///    y.increment(v, 1);
    /// }
    /// assert_eq!(y.percentile(0.0), Ok(0));
    /// assert_eq!(y.percentile(0.5), Ok(49));
    /// assert_eq!(y.percentile(1.0), Ok(99));
    /// ```
    pub fn percentile(&self, percentile: f64) -> Result<u64, HistogramError> {
        let total = self.total_count();
        if total == 0 {
            Err(HistogramError::Empty)
        } else {
            let mut need = (percentile * total as f64).ceil() as u64;
            if need == 0 {
                need = 1;
            }
            let mut have = 0;
            for i in 0..self.index.len() {
                let count = self.index[i].load(Ordering::Relaxed);
                if have + count >= need {
                    let index = i * 100;
                    for j in index..(index + 100) {
                        have += u64::from(self.buckets[j].load(Ordering::Relaxed));
                        if have >= need {
                            return Ok(self.get_value(j).unwrap());
                        }
                    }
                }
                have += count;
            }
            Ok(self.max.load(Ordering::Relaxed))
        }
    }

    /// Return the number of samples which were too high to store in a `Bucket`
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_datastructures::*;
    ///
    /// let x = Histogram::<AtomicU64>::new(100, 2, None, None);
    /// x.increment(420, 1);
    /// assert_eq!(x.too_high(), 1);
    /// ```
    pub fn too_high(&self) -> u64 {
        self.too_high.load(Ordering::Relaxed)
    }

    /// Returns the approximate mean of all values in the `Histogram`
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_datastructures::*;
    ///
    /// let x = Histogram::<AtomicU64>::new(100, 2, None, None);
    /// for v in 0..100 {
    ///    x.increment(v, 1);
    /// }
    /// assert_eq!(x.mean(), Ok(49));
    /// ```
    pub fn mean(&self) -> Result<u64, HistogramError> {
        let total_count = self.total_count();
        if total_count > 0 {
            let mut result = 0;
            for bucket in self.into_iter() {
                result += u64::from(bucket.count) * bucket.value;
            }
            Ok(result / total_count)
        } else {
            Err(HistogramError::Empty)
        }
    }

    /// Returns the nominal value of the mode of the `Histogram`
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_datastructures::*;
    ///
    /// let x = Histogram::<AtomicU64>::new(100, 2, None, None);
    /// for v in 0..100 {
    ///    x.increment(v, v);
    /// }
    /// assert_eq!(x.mode(), Ok(99));
    /// ```
    pub fn mode(&self) -> Result<u64, HistogramError> {
        let total_count = self.total_count();
        if total_count > 0 {
            let mut count = 0;
            let mut value = 0;
            for bucket in self.into_iter() {
                if u64::from(bucket.count()) > count {
                    count = u64::from(bucket.count());
                    value = bucket.value();
                }
            }
            Ok(value)
        } else {
            Err(HistogramError::Empty)
        }
    }
}

pub struct Iter<'a, T>
where
    T: Atomic + Unsigned + SaturatingArithmetic + Default,
    <T as Atomic>::Primitive: Default + PartialEq + Copy,
{
    inner: &'a Histogram<T>,
    index: usize,
}

impl<'a, T> Iter<'a, T>
where
    T: Atomic + Unsigned + SaturatingArithmetic + Default,
    <T as Atomic>::Primitive: Default + PartialEq + Copy,
{
    fn new(inner: &'a Histogram<T>) -> Iter<'a, T> {
        Iter { inner, index: 0 }
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: Atomic + Unsigned + SaturatingArithmetic + Default,
    <T as Atomic>::Primitive: Default + PartialEq + Copy,
    u64: From<<T as Atomic>::Primitive>,
{
    type Item = Bucket<<T as Atomic>::Primitive>;

    fn next(&mut self) -> Option<Bucket<<T as Atomic>::Primitive>> {
        let bucket = self.inner.get_bucket(self.index);
        self.index += 1;
        bucket
    }
}

impl<'a, T> IntoIterator for &'a Histogram<T>
where
    T: Atomic + Unsigned + SaturatingArithmetic + Default,
    <T as Atomic>::Primitive: Default + PartialEq + Copy,
    u64: From<<T as Atomic>::Primitive>,
{
    type Item = Bucket<<T as Atomic>::Primitive>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_latched() {
        let h = Histogram::<AtomicU64>::new(100, 3, None, None);
        assert_eq!(h.total_count(), 0);
        for i in 1..=100 {
            let _ = h.increment(i, 1);
            assert_eq!(h.total_count(), i);
        }
        assert_eq!(h.percentile(0.0), Ok(1));
        assert_eq!(h.percentile(0.05), Ok(5));
        assert_eq!(h.percentile(0.1), Ok(10));
        assert_eq!(h.percentile(0.25), Ok(25));
        assert_eq!(h.percentile(0.50), Ok(50));
        assert_eq!(h.percentile(0.75), Ok(75));
        assert_eq!(h.percentile(0.90), Ok(90));
        assert_eq!(h.percentile(0.95), Ok(95));
        assert_eq!(h.percentile(0.99), Ok(99));
        assert_eq!(h.percentile(0.999), Ok(100));
        assert_eq!(h.percentile(1.0), Ok(100));
        h.clear();
        assert_eq!(h.percentile(0.0), Err(HistogramError::Empty));
        assert_eq!(h.total_count(), 0);
        assert_eq!(h.size(), 936);
    }

    #[test]
    fn size() {
        let h = Histogram::<AtomicU8>::new(1_000_000_000, 3, None, None);
        assert_eq!(h.size() / 1024, 6); // ~6KB

        let h = Histogram::<AtomicU16>::new(1_000_000_000, 3, None, None);
        assert_eq!(h.size() / 1024, 13); // ~13KB

        let h = Histogram::<AtomicU32>::new(1_000_000_000, 3, None, None);
        assert_eq!(h.size() / 1024, 25); // ~25KB

        let h = Histogram::<AtomicU32>::new(60_000_000_000, 3, None, None);
        assert_eq!(h.size() / 1024, 31); // ~31KB

        let h = Histogram::<AtomicU64>::new(1_000_000_000, 3, None, None);
        assert_eq!(h.size() / 1024, 50); // ~50KB

        let h =
            Histogram::<AtomicU64>::new(1_000_000_000, 3, Some(<Duration>::from_millis(1)), None);
        assert_eq!(h.size() / 1024, 50); // ~50KB

        let h = Histogram::<AtomicU64>::new(
            1_000_000_000,
            3,
            Some(<Duration>::from_millis(1)),
            Some(60),
        );
        assert!(h.size() / 1024 >= 52); // ~52KB
        assert!(h.size() / 1024 <= 53); // ~52KB
    }

    #[test]
    fn basic_moving() {
        let h = Histogram::<AtomicU64>::new(100, 3, Some(<Duration>::from_millis(1)), None);
        assert_eq!(h.total_count(), 0);
        for i in 1..100 {
            let _ = h.increment(i, 1);
            assert_eq!(h.total_count(), 1);
            assert_eq!(h.percentile(0.0), Ok(i));
            assert_eq!(h.percentile(1.0), Ok(i));
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(h.total_count(), 0);
    }

    #[test]
    fn basic_capacity() {
        let h = Histogram::<AtomicU64>::new(100, 3, None, Some(1));
        assert_eq!(h.total_count(), 0);
        for i in 1..100 {
            let _ = h.increment(i, 1);
            assert_eq!(h.total_count(), 1);
            assert_eq!(h.percentile(0.0), Ok(i));
            assert_eq!(h.percentile(1.0), Ok(i));
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(h.total_count(), 1);
    }

    #[test]
    fn basic_moving_capacity() {
        let h = Histogram::<AtomicU64>::new(100, 3, Some(<Duration>::from_millis(1)), Some(1));
        assert_eq!(h.total_count(), 0);
        for i in 1..100 {
            let _ = h.increment(i, 1);
            assert_eq!(h.total_count(), 1);
            assert_eq!(h.percentile(0.0), Ok(i));
            assert_eq!(h.percentile(1.0), Ok(i));
        }
        assert_eq!(h.total_count(), 1);
        std::thread::sleep(Duration::from_millis(1));
        assert_eq!(h.total_count(), 0);
    }

    #[test]
    fn bounds_latched() {
        let h = Histogram::<AtomicU64>::new(100, 3, None, None);
        assert_eq!(h.total_count(), 0);
        for i in 1..=100 {
            let _ = h.increment(1_000_000, 1);
            assert_eq!(h.total_count(), i);
        }
        h.clear();
        assert_eq!(h.total_count(), 0);
    }

    #[test]
    fn latched_incr_decr() {
        let h = Histogram::<AtomicU64>::new(100, 3, None, None);
        assert_eq!(h.total_count(), 0);
        h.increment(1, 1);
        assert_eq!(h.total_count(), 1);
        h.increment(2, 1);
        assert_eq!(h.total_count(), 2);
        h.decrement(2, 1);
        assert_eq!(h.total_count(), 1);
        h.decrement(1, 1);
        assert_eq!(h.total_count(), 0);

        h.increment(1_000_000, 1);
        assert_eq!(h.total_count(), 1);
        h.decrement(1_000_000, 1);
        assert_eq!(h.total_count(), 0);
    }

    #[test]
    fn bounds_moving() {
        let h = Histogram::<AtomicU64>::new(100, 3, Some(<Duration>::from_millis(20)), None);
        assert_eq!(h.total_count(), 0);
        for _ in 1..100 {
            let _ = h.increment(1_000_000, 1);
            assert_eq!(h.total_count(), 1);
            assert_eq!(h.percentile(0.0), Ok(100));
            assert_eq!(h.percentile(1.0), Ok(100));
            std::thread::sleep(Duration::from_millis(20));
        }
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(h.percentile(0.0), Err(HistogramError::Empty));
        assert_eq!(h.percentile(1.0), Err(HistogramError::Empty));
        assert_eq!(h.total_count(), 0);
    }

    #[test]
    fn bounds_moving_capacity() {
        let h = Histogram::<AtomicU64>::new(100, 3, Some(<Duration>::from_millis(1)), Some(1));
        assert_eq!(h.total_count(), 0);
        for _ in 1..100 {
            let _ = h.increment(1_000_000, 1);
            assert_eq!(h.total_count(), 1);
            assert_eq!(h.percentile(0.0), Ok(100));
            assert_eq!(h.percentile(1.0), Ok(100));
        }
        std::thread::sleep(Duration::from_millis(2));
        assert_eq!(h.percentile(0.0), Err(HistogramError::Empty));
        assert_eq!(h.percentile(1.0), Err(HistogramError::Empty));
        assert_eq!(h.total_count(), 0);
    }
}
