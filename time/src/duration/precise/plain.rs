// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Temporal quantification.
//!
//! Example:
//!
//! ```
//! use rustcommon_time::Duration;
//!
//! let five_seconds = Duration::new(5, 0);
//! // both declarations are equivalent
//! assert_eq!(Duration::new(5, 0), Duration::from_secs(5));
//! ```

use core::fmt;
use core::iter::Sum;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::*;

/// A `Duration` type to represent a span of time, typically used for system
/// timeouts.
///
/// Each `Duration` is composed of a whole number of seconds and a fractional part
/// represented in nanoseconds. If the underlying system does not support
/// nanosecond-level precision, APIs binding a system timeout will typically round up
/// the number of nanoseconds.
///
/// [`Duration`]s implement many common traits, including [`Add`], [`Sub`], and other
/// [`ops`] traits. It implements [`Default`] by returning a zero-length `Duration`.
///
/// [`ops`]: core::ops
///
/// # Examples
///
/// ```
/// use rustcommon_time::Duration;
///
/// let five_seconds = Duration::new(5, 0);
/// let five_seconds_and_five_nanos = five_seconds + Duration::new(0, 5);
///
/// assert_eq!(five_seconds_and_five_nanos.as_secs(), 5);
/// assert_eq!(five_seconds_and_five_nanos.subsec_nanos(), 5);
///
/// let ten_millis = Duration::from_millis(10);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Duration {
    pub(crate) nanos: u64,
}

impl Duration {
    /// The duration of one second.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// assert_eq!(Duration::SECOND, Duration::from_secs(1));
    /// ```
    pub const SECOND: Duration = Duration::from_nanos(NANOS_PER_SEC);

    /// The duration of one millisecond.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// assert_eq!(Duration::MILLISECOND, Duration::from_millis(1));
    /// ```
    pub const MILLISECOND: Duration = Duration::from_nanos(NANOS_PER_MILLI);

    /// The duration of one microsecond.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// assert_eq!(Duration::MICROSECOND, Duration::from_micros(1));
    /// ```
    pub const MICROSECOND: Duration = Duration::from_nanos(NANOS_PER_MICRO);

    /// The duration of one nanosecond.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// assert_eq!(Duration::NANOSECOND, Duration::from_nanos(1));
    /// ```
    pub const NANOSECOND: Duration = Duration::from_nanos(1);

    /// A duration of zero time.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let duration = Duration::ZERO;
    /// assert!(duration.is_zero());
    /// assert_eq!(duration.as_nanos(), 0);
    /// ```
    pub const ZERO: Duration = Duration::from_nanos(0);

    /// The maximum duration.
    ///
    /// It is roughly equal to a duration of 584.942 years.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// assert_eq!(Duration::MAX, Duration::from_nanos(u64::MAX));
    /// ```
    pub const MAX: Duration = Duration::from_nanos(u64::MAX);

    /// Creates a new `Duration` from the specified number of whole seconds and
    /// additional nanoseconds.
    ///
    /// If the number of nanoseconds is greater than 1 billion (the number of
    /// nanoseconds in a second), then it will carry over into the seconds
    /// provided.
    ///
    /// # Panics
    ///
    /// This constructor will panic if the total duration exceeds
    /// `Duration::MAX`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let five_seconds = Duration::new(5, 0);
    /// ```
    #[inline]
    pub fn new(secs: u64, nanos: u32) -> Duration {
        let secs_ns = secs
            .checked_mul(NANOS_PER_SEC)
            .expect("number of seconds caused overflow");
        let nanos = secs_ns
            .checked_add(nanos as u64)
            .expect("total duration caused overflow");
        Duration { nanos }
    }

    /// Creates a new `Duration` from the specified number of whole seconds.
    ///
    /// # Panics
    /// This constructor will panic if `secs` overflows `Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let duration = Duration::from_secs(5);
    ///
    /// assert_eq!(5, duration.as_secs());
    /// assert_eq!(0, duration.subsec_nanos());
    /// ```
    #[inline]
    pub fn from_secs(secs: u64) -> Duration {
        let nanos = secs
            .checked_mul(NANOS_PER_SEC)
            .expect("total duration caused overflow");
        Duration { nanos }
    }

    /// Creates a new `Duration` from the specified number of milliseconds.
    ///
    /// # Panics
    /// This constructor will panic if `millis` overflows `Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let duration = Duration::from_millis(2569);
    ///
    /// assert_eq!(2, duration.as_secs());
    /// assert_eq!(569_000_000, duration.subsec_nanos());
    /// ```
    #[inline]
    pub fn from_millis(millis: u64) -> Duration {
        let nanos = millis
            .checked_mul(NANOS_PER_MILLI)
            .expect("total duration caused overflow");
        Duration { nanos }
    }

    /// Creates a new `Duration` from the specified number of microseconds.
    ///
    /// # Panics
    /// This constructor will panic if `micros` overflows `Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let duration = Duration::from_micros(1_000_002);
    ///
    /// assert_eq!(1, duration.as_secs());
    /// assert_eq!(2000, duration.subsec_nanos());
    /// ```
    #[inline]
    pub fn from_micros(micros: u64) -> Duration {
        let nanos = micros
            .checked_mul(NANOS_PER_MICRO)
            .expect("total duration caused overflow");
        Duration { nanos }
    }

    /// Creates a new `Duration` from the specified number of nanoseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let duration = Duration::from_nanos(1_000_000_123);
    ///
    /// assert_eq!(1, duration.as_secs());
    /// assert_eq!(123, duration.subsec_nanos());
    /// ```
    #[inline]
    pub const fn from_nanos(nanos: u64) -> Duration {
        Duration { nanos }
    }

    /// Returns true if this `Duration` spans no time.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// assert!(Duration::ZERO.is_zero());
    /// assert!(Duration::new(0, 0).is_zero());
    /// assert!(Duration::from_nanos(0).is_zero());
    /// assert!(Duration::from_secs(0).is_zero());
    ///
    /// assert!(!Duration::new(1, 1).is_zero());
    /// assert!(!Duration::from_nanos(1).is_zero());
    /// assert!(!Duration::from_secs(1).is_zero());
    /// ```
    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.nanos == 0
    }

    /// Returns the number of _whole_ seconds contained by this `Duration`.
    ///
    /// The returned value does not include the fractional (nanosecond) part of
    /// the duration, which can be obtained using [`subsec_nanos`].
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    /// assert_eq!(duration.as_secs(), 5);
    /// ```
    ///
    /// To determine the total number of seconds represented by the `Duration`,
    /// use `as_secs` in combination with [`subsec_nanos`]:
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    ///
    /// assert_eq!(5.730023852,
    ///            duration.as_secs() as f64
    ///            + duration.subsec_nanos() as f64 * 1e-9);
    /// ```
    ///
    /// [`subsec_nanos`]: Duration::subsec_nanos
    #[inline]
    pub const fn as_secs(&self) -> u64 {
        self.nanos / NANOS_PER_SEC
    }

    /// Returns the fractional part of this `Duration`, in whole milliseconds.
    ///
    /// This method does **not** return the length of the duration when
    /// represented by milliseconds. The returned number always represents a
    /// fractional portion of a second (i.e., it is less than one thousand).
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let duration = Duration::from_millis(5432);
    /// assert_eq!(duration.as_secs(), 5);
    /// assert_eq!(duration.subsec_millis(), 432);
    /// ```
    #[inline]
    pub const fn subsec_millis(&self) -> u32 {
        ((self.nanos % NANOS_PER_SEC) / NANOS_PER_MILLI) as u32
    }

    /// Returns the fractional part of this `Duration`, in whole microseconds.
    ///
    /// This method does **not** return the length of the duration when
    /// represented by microseconds. The returned number always represents a
    /// fractional portion of a second (i.e., it is less than one million).
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let duration = Duration::from_micros(1_234_567);
    /// assert_eq!(duration.as_secs(), 1);
    /// assert_eq!(duration.subsec_micros(), 234_567);
    /// ```
    #[inline]
    pub const fn subsec_micros(&self) -> u32 {
        ((self.nanos % NANOS_PER_SEC) / NANOS_PER_MICRO) as u32
    }

    /// Returns the fractional part of this `Duration`, in nanoseconds.
    ///
    /// This method does **not** return the length of the duration when
    /// represented by nanoseconds. The returned number always represents a
    /// fractional portion of a second (i.e., it is less than one billion).
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let duration = Duration::from_millis(5010);
    /// assert_eq!(duration.as_secs(), 5);
    /// assert_eq!(duration.subsec_nanos(), 10_000_000);
    /// ```
    #[inline]
    pub const fn subsec_nanos(&self) -> u32 {
        (self.nanos % NANOS_PER_SEC) as u32
    }

    /// Returns the total number of whole milliseconds contained by this
    /// `Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    /// assert_eq!(duration.as_millis(), 5730);
    /// ```
    #[inline]
    pub const fn as_millis(&self) -> u128 {
        (self.nanos / NANOS_PER_MILLI) as u128
    }

    /// Returns the total number of whole microseconds contained by this
    /// `Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    /// assert_eq!(duration.as_micros(), 5730023);
    /// ```
    #[inline]
    pub const fn as_micros(&self) -> u128 {
        (self.nanos / NANOS_PER_MICRO) as u128
    }

    /// Returns the total number of nanoseconds contained by this `Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    /// assert_eq!(duration.as_nanos(), 5730023852);
    /// ```
    #[inline]
    pub const fn as_nanos(&self) -> u128 {
        self.nanos as u128
    }

    /// Checked `Duration` addition. Computes `self + other`, returning [`None`]
    /// if overflow occurred.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 0).checked_add(Duration::new(0, 1)), Some(Duration::new(0, 1)));
    /// assert_eq!(Duration::new(1, 0).checked_add(Duration::new(u64::MAX / 1_000_000_000, 0)), None);
    /// ```
    #[inline]
    pub const fn checked_add(self, rhs: Duration) -> Option<Duration> {
        if let Some(nanos) = self.nanos.checked_add(rhs.nanos) {
            Some(Duration { nanos })
        } else {
            None
        }
    }

    /// Saturating `Duration` addition. Computes `self + other`, returning
    /// [`Duration::MAX`] if overflow occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 0).saturating_add(Duration::new(0, 1)), Duration::new(0, 1));
    /// assert_eq!(Duration::new(1, 0).saturating_add(Duration::new(u64::MAX / 1_000_000_000, 0)), Duration::MAX);
    /// ```
    #[inline]
    pub const fn saturating_add(self, rhs: Duration) -> Duration {
        match self.checked_add(rhs) {
            Some(res) => res,
            None => Duration::MAX,
        }
    }

    /// Checked `Duration` subtraction. Computes `self - other`, returning
    /// [`None`] if the result would be negative or if overflow occurred.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 1).checked_sub(Duration::new(0, 0)), Some(Duration::new(0, 1)));
    /// assert_eq!(Duration::new(0, 0).checked_sub(Duration::new(0, 1)), None);
    /// ```
    #[inline]
    pub const fn checked_sub(self, rhs: Duration) -> Option<Duration> {
        if let Some(nanos) = self.nanos.checked_sub(rhs.nanos) {
            Some(Duration { nanos })
        } else {
            None
        }
    }

    /// Saturating `Duration` subtraction. Computes `self - other`, returning
    /// [`Duration::ZERO`] if the result would be negative or if overflow
    /// occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 1).saturating_sub(Duration::new(0, 0)), Duration::new(0, 1));
    /// assert_eq!(Duration::new(0, 0).saturating_sub(Duration::new(0, 1)), Duration::ZERO);
    /// ```
    #[inline]
    pub const fn saturating_sub(self, rhs: Duration) -> Duration {
        match self.checked_sub(rhs) {
            Some(res) => res,
            None => Duration::ZERO,
        }
    }

    /// Checked `Duration` multiplication. Computes `self * other`, returning
    /// [`None`] if overflow occurred.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 500_000_001).checked_mul(2), Some(Duration::new(1, 2)));
    /// assert_eq!(Duration::new(u64::MAX / 1_000_000_000, 0).checked_mul(2), None);
    /// ```
    #[inline]
    pub const fn checked_mul(self, rhs: u32) -> Option<Duration> {
        if let Some(nanos) = self.nanos.checked_mul(rhs as u64) {
            Some(Duration { nanos })
        } else {
            None
        }
    }

    /// Saturating `Duration` multiplication. Computes `self * other`, returning
    /// [`Duration::MAX`] if overflow occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 500_000_001).saturating_mul(2), Duration::new(1, 2));
    /// assert_eq!(Duration::new(u64::MAX / 1_000_000_000, 0).saturating_mul(2), Duration::MAX);
    /// ```
    #[inline]
    pub const fn saturating_mul(self, rhs: u32) -> Duration {
        match self.checked_mul(rhs) {
            Some(res) => res,
            None => Duration::MAX,
        }
    }

    /// Checked `Duration` division. Computes `self / other`, returning [`None`]
    /// if `other == 0`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// assert_eq!(Duration::new(2, 0).checked_div(2), Some(Duration::new(1, 0)));
    /// assert_eq!(Duration::new(1, 0).checked_div(2), Some(Duration::new(0, 500_000_000)));
    /// assert_eq!(Duration::new(2, 0).checked_div(0), None);
    /// ```
    #[inline]
    pub const fn checked_div(self, rhs: u32) -> Option<Duration> {
        if rhs != 0 {
            let nanos = self.nanos / rhs as u64;
            Some(Duration { nanos })
        } else {
            None
        }
    }

    /// Returns the number of seconds contained by this `Duration` as `f64`.
    ///
    /// The returned value does include the fractional (nanosecond) part of the
    /// duration.
    ///
    /// # Examples
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let dur = Duration::new(2, 700_000_000);
    /// assert_eq!(dur.as_secs_f64(), 2.7);
    /// ```
    #[inline]
    pub fn as_secs_f64(&self) -> f64 {
        (self.nanos as f64) / (NANOS_PER_SEC as f64)
    }

    /// Returns the number of seconds contained by this `Duration` as `f32`.
    ///
    /// The returned value does include the fractional (nanosecond) part of the
    /// duration.
    ///
    /// # Examples
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let dur = Duration::new(2, 700_000_000);
    /// assert_eq!(dur.as_secs_f32(), 2.7);
    /// ```
    #[inline]
    pub fn as_secs_f32(&self) -> f32 {
        (self.nanos as f32) / (NANOS_PER_SEC as f32)
    }

    /// Creates a new `Duration` from the specified number of seconds
    /// represented as `f64`.
    ///
    /// # Panics
    /// This constructor will panic if `secs` is not finite, negative or
    /// overflows `Duration`.
    ///
    /// # Examples
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let dur = Duration::from_secs_f64(2.7);
    /// assert_eq!(dur, Duration::new(2, 700_000_000));
    /// ```
    #[inline]
    pub fn from_secs_f64(secs: f64) -> Duration {
        let nanos = secs * (NANOS_PER_SEC as f64);
        if !nanos.is_finite() {
            panic!("got non-finite value when converting float to duration");
        }
        if nanos > u64::MAX as f64 {
            panic!("overflow when converting float to duration");
        }
        if nanos < 0.0 {
            panic!("underflow when converting float to duration");
        }
        Duration {
            nanos: nanos as u64,
        }
    }

    /// Creates a new `Duration` from the specified number of seconds
    /// represented as `f32`.
    ///
    /// # Panics
    /// This constructor will panic if `secs` is not finite, negative or
    /// overflows `Duration`.
    ///
    /// # Examples
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let dur = Duration::from_secs_f32(2.7);
    /// assert_eq!(dur, Duration::new(2, 700_000_000));
    /// ```
    #[inline]
    pub fn from_secs_f32(secs: f32) -> Duration {
        let nanos = secs * (NANOS_PER_SEC as f32);
        if !nanos.is_finite() {
            panic!("got non-finite value when converting float to duration");
        }
        if nanos > u64::MAX as f32 {
            panic!("overflow when converting float to duration");
        }
        if nanos < 0.0 {
            panic!("underflow when converting float to duration");
        }
        Duration {
            nanos: nanos as u64,
        }
    }

    /// Multiplies `Duration` by `f64`.
    ///
    /// # Panics
    /// This method will panic if result is not finite, negative or overflows
    /// `Duration`.
    ///
    /// # Examples
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let dur = Duration::new(2, 700_000_000);
    /// assert_eq!(dur.mul_f64(3.14), Duration::new(8, 478_000_000));
    /// assert_eq!(dur.mul_f64(3.14e5), Duration::new(847_800, 0));
    /// ```
    #[inline]
    pub fn mul_f64(self, rhs: f64) -> Duration {
        Duration::from_secs_f64(rhs * self.as_secs_f64())
    }

    /// Multiplies `Duration` by `f32`.
    ///
    /// # Panics
    /// This method will panic if result is not finite, negative or overflows
    /// `Duration`.
    ///
    /// # Examples
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let dur = Duration::new(2, 700_000_000);
    /// // note that due to rounding errors result is slightly different
    /// // from 8.478 and 847800.0
    /// assert_eq!(dur.mul_f32(3.14), Duration::new(8, 478_000_640));
    /// assert_eq!(dur.mul_f32(3.14e5), Duration::new(847799, 969_120_256));
    /// ```
    #[inline]
    pub fn mul_f32(self, rhs: f32) -> Duration {
        Duration::from_secs_f32(rhs * self.as_secs_f32())
    }

    /// Divide `Duration` by `f64`.
    ///
    /// # Panics
    /// This method will panic if result is not finite, negative or overflows
    /// `Duration`.
    ///
    /// # Examples
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let dur = Duration::new(2, 700_000_000);
    /// assert_eq!(dur.div_f64(3.14), Duration::new(0, 859_872_611));
    /// // note that truncation is used, not rounding
    /// assert_eq!(dur.div_f64(3.14e5), Duration::new(0, 8_598));
    /// ```
    #[inline]
    pub fn div_f64(self, rhs: f64) -> Duration {
        Duration::from_secs_f64(self.as_secs_f64() / rhs)
    }

    /// Divide `Duration` by `f32`.
    ///
    /// # Panics
    /// This method will panic if result is not finite, negative or overflows
    /// `Duration`.
    ///
    /// # Examples
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let dur = Duration::new(2, 700_000_000);
    /// // note that due to rounding errors result is slightly
    /// // different from 0.859_872_611
    /// assert_eq!(dur.div_f32(3.14), Duration::new(0, 859_872_576));
    /// // note that truncation is used, not rounding
    /// assert_eq!(dur.div_f32(3.14e5), Duration::new(0, 8_598));
    /// ```
    #[inline]
    pub fn div_f32(self, rhs: f32) -> Duration {
        Duration::from_secs_f32(self.as_secs_f32() / rhs)
    }

    /// Divide `Duration` by `Duration` and return `f64`.
    ///
    /// # Examples
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let dur1 = Duration::new(2, 700_000_000);
    /// let dur2 = Duration::new(5, 400_000_000);
    /// assert_eq!(dur1.div_duration_f64(dur2), 0.5);
    /// ```
    #[inline]
    pub fn div_duration_f64(self, rhs: Duration) -> f64 {
        self.as_secs_f64() / rhs.as_secs_f64()
    }

    /// Divide `Duration` by `Duration` and return `f32`.
    ///
    /// # Examples
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let dur1 = Duration::new(2, 700_000_000);
    /// let dur2 = Duration::new(5, 400_000_000);
    /// assert_eq!(dur1.div_duration_f32(dur2), 0.5);
    /// ```
    #[inline]
    pub fn div_duration_f32(self, rhs: Duration) -> f32 {
        self.as_secs_f32() / rhs.as_secs_f32()
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Duration) -> Duration {
        self.checked_add(rhs)
            .expect("overflow when adding durations")
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Duration) -> Duration {
        self.checked_sub(rhs)
            .expect("overflow when subtracting durations")
    }
}

impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

impl Mul<u32> for Duration {
    type Output = Duration;

    fn mul(self, rhs: u32) -> Duration {
        self.checked_mul(rhs)
            .expect("overflow when multiplying duration by scalar")
    }
}

impl Mul<Duration> for u32 {
    type Output = Duration;

    fn mul(self, rhs: Duration) -> Duration {
        rhs * self
    }
}

impl MulAssign<u32> for Duration {
    fn mul_assign(&mut self, rhs: u32) {
        *self = *self * rhs;
    }
}

impl Div<u32> for Duration {
    type Output = Duration;

    fn div(self, rhs: u32) -> Duration {
        self.checked_div(rhs)
            .expect("divide by zero error when dividing duration by scalar")
    }
}

impl DivAssign<u32> for Duration {
    fn div_assign(&mut self, rhs: u32) {
        *self = *self / rhs;
    }
}

macro_rules! sum_durations {
    ($iter:expr) => {{
        let mut nanos: u64 = 0;

        for entry in $iter {
            nanos = nanos
                .checked_add(entry.nanos as u64)
                .expect("overflow in iter::sum over durations");
        }
        Duration { nanos }
    }};
}

impl Sum for Duration {
    fn sum<I: Iterator<Item = Duration>>(iter: I) -> Duration {
        sum_durations!(iter)
    }
}

impl<'a> Sum<&'a Duration> for Duration {
    fn sum<I: Iterator<Item = &'a Duration>>(iter: I) -> Duration {
        sum_durations!(iter)
    }
}

impl fmt::Debug for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        /// Formats a floating point number in decimal notation.
        ///
        /// The number is given as the `integer_part` and a fractional part.
        /// The value of the fractional part is `fractional_part / divisor`. So
        /// `integer_part` = 3, `fractional_part` = 12 and `divisor` = 100
        /// represents the number `3.012`. Trailing zeros are omitted.
        ///
        /// `divisor` must not be above 100_000_000. It also should be a power
        /// of 10, everything else doesn't make sense. `fractional_part` has
        /// to be less than `10 * divisor`!
        fn fmt_decimal(
            f: &mut fmt::Formatter<'_>,
            mut integer_part: u64,
            mut fractional_part: u64,
            mut divisor: u64,
        ) -> fmt::Result {
            // Encode the fractional part into a temporary buffer. The buffer
            // only need to hold 9 elements, because `fractional_part` has to
            // be smaller than 10^9. The buffer is prefilled with '0' digits
            // to simplify the code below.
            let mut buf = [b'0'; 9];

            // The next digit is written at this position
            let mut pos = 0;

            // We keep writing digits into the buffer while there are non-zero
            // digits left and we haven't written enough digits yet.
            while fractional_part > 0 && pos < f.precision().unwrap_or(9) {
                // Write new digit into the buffer
                buf[pos] = b'0' + (fractional_part / divisor) as u8;

                fractional_part %= divisor;
                divisor /= 10;
                pos += 1;
            }

            // If a precision < 9 was specified, there may be some non-zero
            // digits left that weren't written into the buffer. In that case we
            // need to perform rounding to match the semantics of printing
            // normal floating point numbers. However, we only need to do work
            // when rounding up. This happens if the first digit of the
            // remaining ones is >= 5.
            if fractional_part > 0 && fractional_part >= divisor * 5 {
                // Round up the number contained in the buffer. We go through
                // the buffer backwards and keep track of the carry.
                let mut rev_pos = pos;
                let mut carry = true;
                while carry && rev_pos > 0 {
                    rev_pos -= 1;

                    // If the digit in the buffer is not '9', we just need to
                    // increment it and can stop then (since we don't have a
                    // carry anymore). Otherwise, we set it to '0' (overflow)
                    // and continue.
                    if buf[rev_pos] < b'9' {
                        buf[rev_pos] += 1;
                        carry = false;
                    } else {
                        buf[rev_pos] = b'0';
                    }
                }

                // If we still have the carry bit set, that means that we set
                // the whole buffer to '0's and need to increment the integer
                // part.
                if carry {
                    integer_part += 1;
                }
            }

            // Determine the end of the buffer: if precision is set, we just
            // use as many digits from the buffer (capped to 9). If it isn't
            // set, we only use all digits up to the last non-zero one.
            let end = f.precision().map(|p| core::cmp::min(p, 9)).unwrap_or(pos);

            // If we haven't emitted a single fractional digit and the precision
            // wasn't set to a non-zero value, we don't print the decimal point.
            if end == 0 {
                write!(f, "{}", integer_part)
            } else {
                // SAFETY: We are only writing ASCII digits into the buffer and
                // it was initialized with '0's, so it contains valid UTF8.
                let s = unsafe { core::str::from_utf8_unchecked(&buf[..end]) };

                // If the user request a precision > 9, we pad '0's at the end.
                let w = f.precision().unwrap_or(pos);
                write!(f, "{}.{:0<width$}", integer_part, s, width = w)
            }
        }

        // Print leading '+' sign if requested
        if f.sign_plus() {
            write!(f, "+")?;
        }

        let secs = self.nanos / NANOS_PER_SEC;
        let nanos = self.nanos % NANOS_PER_SEC;

        if secs > 0 {
            fmt_decimal(f, secs, nanos, NANOS_PER_SEC / 10)?;
            f.write_str("s")
        } else if nanos >= NANOS_PER_MILLI {
            fmt_decimal(
                f,
                (nanos / NANOS_PER_MILLI) as u64,
                nanos % NANOS_PER_MILLI,
                NANOS_PER_MILLI / 10,
            )?;
            f.write_str("ms")
        } else if nanos >= NANOS_PER_MICRO {
            fmt_decimal(
                f,
                (nanos / NANOS_PER_MICRO) as u64,
                nanos % NANOS_PER_MICRO,
                NANOS_PER_MICRO / 10,
            )?;
            f.write_str("µs")
        } else {
            fmt_decimal(f, self.nanos as u64, 0, 1)?;
            f.write_str("ns")
        }
    }
}
