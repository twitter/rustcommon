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

/// A `CoarseDuration` type to represent a span of time with a resolution of one
/// second in a 32 bit value. This is appropriate for representing things such
/// as object expiration in a cache.
///
/// Each `CoarseDuration` is composed of a whole number of seconds.
///
/// [`CoarseDuration`]s implement many common traits, including [`Add`],
/// [`Sub`], and other [`ops`] traits. It implements [`Default`] by returning a
/// zero-length `CoarseDuration`.
///
/// [`ops`]: core::ops
///
/// # Examples
///
/// ```
/// use rustcommon_time::CoarseDuration;
///
/// let five_seconds = CoarseDuration::new(5);
///
/// assert_eq!(five_seconds.as_secs(), 5);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CoarseDuration {
    pub(crate) secs: u32,
}

impl CoarseDuration {
    /// The duration of one second.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// assert_eq!(CoarseDuration::SECOND, CoarseDuration::from_secs(1));
    /// ```
    pub const SECOND: Self = Self::from_secs(1);

    /// A duration of zero time.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// let duration = CoarseDuration::ZERO;
    /// assert!(duration.is_zero());
    /// assert_eq!(duration.as_nanos(), 0);
    /// ```
    pub const ZERO: Self = Self::from_secs(0);

    /// The maximum duration.
    ///
    /// It is roughly equal to a duration of 136.192 years.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// assert_eq!(CoarseDuration::MAX, CoarseDuration::new(u32::MAX));
    /// ```
    pub const MAX: Self = Self::from_secs(u32::MAX);

    /// Creates a new `CoarseDuration` from the specified number of whole
    /// seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// let five_seconds = CoarseDuration::new(5);
    /// ```
    #[inline]
    pub const fn new(secs: u32) -> Self {
        Self { secs }
    }

    /// Creates a new `Duration` from the specified number of whole seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// let duration = CoarseDuration::from_secs(5);
    ///
    /// assert_eq!(5, duration.as_secs());
    /// ```
    #[inline]
    pub const fn from_secs(secs: u32) -> Self {
        Self { secs }
    }

    /// Returns true if this `CoarseDuration` spans no time.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// assert!(CoarseDuration::ZERO.is_zero());
    /// assert!(CoarseDuration::new(0).is_zero());
    /// assert!(CoarseDuration::from_secs(0).is_zero());
    ///
    /// assert!(!CoarseDuration::new(1).is_zero());
    /// assert!(!CoarseDuration::from_secs(1).is_zero());
    /// ```
    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.secs == 0
    }

    /// Returns the number of seconds contained by this `CoarseDuration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// let duration = CoarseDuration::new(5);
    /// assert_eq!(duration.as_secs(), 5);
    /// ```
    #[inline]
    pub const fn as_secs(&self) -> u32 {
        self.secs
    }

    /// Returns the total number of whole milliseconds contained by this
    /// `CoarseDuration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// let duration = CoarseDuration::new(5);
    /// assert_eq!(duration.as_millis(), 5000);
    /// ```
    #[inline]
    pub const fn as_millis(&self) -> u64 {
        self.secs as u64 * MILLIS_PER_SEC
    }

    /// Returns the total number of whole microseconds contained by this
    /// `CoarseDuration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// let duration = CoarseDuration::new(5);
    /// assert_eq!(duration.as_micros(), 5000000);
    /// ```
    #[inline]
    pub const fn as_micros(&self) -> u64 {
        self.secs as u64 * MICROS_PER_SEC
    }

    /// Returns the total number of nanoseconds contained by this
    /// `CoarseDuration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// let duration = CoarseDuration::new(5);
    /// assert_eq!(duration.as_nanos(), 5000000000);
    /// ```
    #[inline]
    pub const fn as_nanos(&self) -> u64 {
        self.secs as u64 * NANOS_PER_SEC
    }

    /// Checked `CoarseDuration` addition. Computes `self + other`, returning
    /// [`None`] if overflow occurred.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// assert_eq!(CoarseDuration::new(1).checked_add(CoarseDuration::new(2)), Some(CoarseDuration::new(3)));
    /// assert_eq!(CoarseDuration::new(1).checked_add(CoarseDuration::new(u32::MAX)), None);
    /// ```
    #[inline]
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        if let Some(secs) = self.secs.checked_add(rhs.secs) {
            Some(Self { secs })
        } else {
            None
        }
    }

    /// Saturating `CoarseDuration` addition. Computes `self + other`, returning
    /// [`CoarseDuration::MAX`] if overflow occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// assert_eq!(CoarseDuration::new(1).saturating_add(CoarseDuration::new(2)), CoarseDuration::new(3));
    /// assert_eq!(CoarseDuration::new(1).saturating_add(CoarseDuration::new(u32::MAX)), CoarseDuration::MAX);
    /// ```
    #[inline]
    pub const fn saturating_add(self, rhs: Self) -> Self {
        match self.checked_add(rhs) {
            Some(res) => res,
            None => Self::MAX,
        }
    }

    /// Checked `CoarseDuration` subtraction. Computes `self - other`, returning
    /// [`None`] if the result would be negative or if overflow occurred.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// assert_eq!(CoarseDuration::new(3).checked_sub(CoarseDuration::new(2)), Some(CoarseDuration::new(1)));
    /// assert_eq!(CoarseDuration::new(0).checked_sub(CoarseDuration::new(1)), None);
    /// ```
    #[inline]
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        if let Some(secs) = self.secs.checked_sub(rhs.secs) {
            Some(Self { secs })
        } else {
            None
        }
    }

    /// Saturating `CoarseDuration` subtraction. Computes `self - other`,
    /// returning [`CoarseDuration::ZERO`] if the result would be negative or
    /// if overflow occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// assert_eq!(CoarseDuration::new(3).saturating_sub(CoarseDuration::new(2)), CoarseDuration::new(1));
    /// assert_eq!(CoarseDuration::new(0).saturating_sub(CoarseDuration::new(1)), CoarseDuration::ZERO);
    /// ```
    #[inline]
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        match self.checked_sub(rhs) {
            Some(res) => res,
            None => Self::ZERO,
        }
    }

    /// Checked `CoarseDuration` multiplication. Computes `self * other`,
    /// returning [`None`] if overflow occurred.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// assert_eq!(CoarseDuration::new(1).checked_mul(2), Some(CoarseDuration::new(2)));
    /// assert_eq!(CoarseDuration::new(u32::MAX - 1).checked_mul(2), None);
    /// ```
    #[inline]
    pub const fn checked_mul(self, rhs: u32) -> Option<Self> {
        if let Some(secs) = self.secs.checked_mul(rhs) {
            Some(Self { secs })
        } else {
            None
        }
    }

    /// Saturating `CoarseDuration` multiplication. Computes `self * other`,
    /// returning [`Duration::MAX`] if overflow occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// assert_eq!(CoarseDuration::new(1).saturating_mul(2), CoarseDuration::new(2));
    /// assert_eq!(CoarseDuration::new(u32::MAX - 1).saturating_mul(2), CoarseDuration::MAX);
    /// ```
    #[inline]
    pub const fn saturating_mul(self, rhs: u32) -> Self {
        match self.checked_mul(rhs) {
            Some(res) => res,
            None => Self::MAX,
        }
    }

    /// Checked `CoarseDuration` division. Computes `self / other`, returning
    /// [`None`] if `other == 0`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use rustcommon_time::CoarseDuration;
    ///
    /// assert_eq!(CoarseDuration::new(2).checked_div(2), Some(CoarseDuration::new(1)));
    /// assert_eq!(CoarseDuration::new(1).checked_div(2), Some(CoarseDuration::new(0)));
    /// assert_eq!(CoarseDuration::new(2).checked_div(0), None);
    /// ```
    #[inline]
    pub const fn checked_div(self, rhs: u32) -> Option<Self> {
        if rhs != 0 {
            let secs = self.secs / rhs;
            Some(Self { secs })
        } else {
            None
        }
    }
}

impl Add for CoarseDuration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self.checked_add(rhs)
            .expect("overflow when adding durations")
    }
}

impl AddAssign for CoarseDuration {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for CoarseDuration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.checked_sub(rhs)
            .expect("overflow when subtracting durations")
    }
}

impl SubAssign for CoarseDuration {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul<u32> for CoarseDuration {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self {
        self.checked_mul(rhs)
            .expect("overflow when multiplying duration by scalar")
    }
}

impl Mul<CoarseDuration> for u32 {
    type Output = CoarseDuration;

    fn mul(self, rhs: CoarseDuration) -> CoarseDuration {
        rhs * self
    }
}

impl MulAssign<u32> for CoarseDuration {
    fn mul_assign(&mut self, rhs: u32) {
        *self = *self * rhs;
    }
}

impl Div<u32> for CoarseDuration {
    type Output = Self;

    fn div(self, rhs: u32) -> Self {
        self.checked_div(rhs)
            .expect("divide by zero error when dividing duration by scalar")
    }
}

impl DivAssign<u32> for CoarseDuration {
    fn div_assign(&mut self, rhs: u32) {
        *self = *self / rhs;
    }
}

macro_rules! sum_durations {
    ($iter:expr) => {{
        let mut secs: u32 = 0;

        for entry in $iter {
            secs = secs
                .checked_add(entry.secs)
                .expect("overflow in iter::sum over durations");
        }
        CoarseDuration { secs }
    }};
}

impl Sum for CoarseDuration {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        sum_durations!(iter)
    }
}

impl<'a> Sum<&'a Self> for CoarseDuration {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        sum_durations!(iter)
    }
}

impl fmt::Debug for CoarseDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}s", self.secs)
    }
}
