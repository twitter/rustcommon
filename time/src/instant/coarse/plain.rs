#![allow(clippy::needless_doctest_main)]

use crate::*;

use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// A lower precision measurement of a monotonically nondecreasing clock. Opaque
/// and useful only with `CoarseDuration`.
///
/// Unlike `Instant`, `CoarseInstant` uses only 32 bits to represent a
/// measurement from an underlying clock and retains only secondly precision.
///
/// This type is most useful for purposes like object expiration, where finer
/// resolution is not required.
///
/// Example:
///
/// ```no_run
/// use rustcommon_time::{CoarseInstant};
/// use std::thread::sleep;
///
/// fn main() {
///    let now = CoarseInstant::now();
///
///    // we sleep for 2 seconds
///    sleep(core::time::Duration::new(2, 0));
///    // it prints '2'
///    println!("{}", now.elapsed().as_secs());
/// }
/// ```
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoarseInstant {
    pub(crate) secs: u32,
}

impl CoarseInstant {
    /// Returns an instant corresponding to "now".
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::CoarseInstant;
    ///
    /// let now = CoarseInstant::now();
    /// ```
    pub fn now() -> CoarseInstant {
        let precise = Instant::now();
        Self {
            secs: (precise.nanos / NANOS_PER_SEC) as u32,
        }
    }

    /// Returns an instant corresponding to when
    /// `rustcommon_time::clock_refresh` was last called.
    ///
    /// This is useful for when the overhead of measuring the current instant is
    /// too high and an approximate measurement is acceptable. Typically the
    /// clock would be refreshed by a separate thread or outside of any critical
    /// path.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::CoarseInstant;
    ///
    /// let now = CoarseInstant::now();
    /// ```
    pub fn recent() -> CoarseInstant {
        _clock().recent_coarse()
    }

    /// Returns the amount of time elapsed from another instant to this one.
    ///
    /// # Panics
    ///
    /// This function will panic if `earlier` is later than `self`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rustcommon_time::CoarseInstant;
    /// use std::thread::sleep;
    ///
    /// let now = CoarseInstant::now();
    /// sleep(core::time::Duration::new(1, 0));
    /// let new_now = CoarseInstant::now();
    /// println!("{:?}", new_now.duration_since(now));
    /// ```
    pub fn duration_since(&self, earlier: CoarseInstant) -> CoarseDuration {
        let secs = self
            .secs
            .checked_sub(earlier.secs)
            .expect("supplied instant is later than self");
        CoarseDuration { secs }
    }

    /// Returns the amount of time elapsed from another instant to this one,
    /// or None if that instant is later than this one.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rustcommon_time::CoarseInstant;
    /// use std::thread::sleep;
    ///
    /// let now = CoarseInstant::now();
    /// sleep(core::time::Duration::new(1, 0));
    /// let new_now = CoarseInstant::now();
    /// println!("{:?}", new_now.checked_duration_since(now));
    /// println!("{:?}", now.checked_duration_since(new_now)); // None
    /// ```
    pub fn checked_duration_since(&self, earlier: CoarseInstant) -> Option<CoarseDuration> {
        let secs = self.secs.checked_sub(earlier.secs)?;
        Some(CoarseDuration { secs })
    }

    /// Returns the amount of time elapsed from another instant to this one,
    /// or zero duration if that instant is later than this one.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rustcommon_time::CoarseInstant;
    /// use std::thread::sleep;
    ///
    /// let now = CoarseInstant::now();
    /// sleep(core::time::Duration::new(1, 0));
    /// let new_now = CoarseInstant::now();
    /// println!("{:?}", new_now.saturating_duration_since(now));
    /// println!("{:?}", now.saturating_duration_since(new_now)); // 0s
    /// ```
    pub fn saturating_duration_since(&self, earlier: CoarseInstant) -> CoarseDuration {
        let secs = self.secs.saturating_sub(earlier.secs);
        CoarseDuration { secs }
    }

    /// Returns the amount of time elapsed since this instant was created.
    ///
    /// # Panics
    ///
    /// This function may panic if the current time is earlier than this
    /// instant, which is something that can happen if an `CoarseInstant` is
    /// produced synthetically.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::thread::sleep;
    /// use rustcommon_time::{CoarseDuration, CoarseInstant};
    ///
    /// let instant = CoarseInstant::now();
    /// sleep(core::time::Duration::from_secs(3));
    /// assert!(instant.elapsed() >= CoarseDuration::from_secs(3));
    /// ```
    pub fn elapsed(&self) -> CoarseDuration {
        CoarseInstant::now() - *self
    }

    /// Returns `Some(t)` where `t` is the time `self + duration` if `t` can be represented as
    /// `CoarseInstant` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    pub fn checked_add(&self, duration: CoarseDuration) -> Option<CoarseInstant> {
        let secs = self.secs.checked_add(duration.secs)?;
        Some(CoarseInstant { secs })
    }

    /// Returns `Some(t)` where `t` is the time `self - duration` if `t` can be represented as
    /// `CoarseInstant` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    pub fn checked_sub(&self, duration: CoarseDuration) -> Option<CoarseInstant> {
        let secs = self.secs.checked_sub(duration.secs)?;
        Some(CoarseInstant { secs })
    }
}

impl Add<CoarseDuration> for CoarseInstant {
    type Output = CoarseInstant;

    /// # Panics
    ///
    /// This function may panic if the resulting point in time cannot be represented by the
    /// underlying data structure. See [`CoarseInstant::checked_add`] for a version without panic.
    fn add(self, other: CoarseDuration) -> CoarseInstant {
        self.checked_add(other)
            .expect("overflow when adding duration to instant")
    }
}

impl AddAssign<CoarseDuration> for CoarseInstant {
    fn add_assign(&mut self, other: CoarseDuration) {
        *self = *self + other;
    }
}

impl Sub<CoarseDuration> for CoarseInstant {
    type Output = CoarseInstant;

    fn sub(self, other: CoarseDuration) -> CoarseInstant {
        self.checked_sub(other)
            .expect("overflow when subtracting duration from instant")
    }
}

impl SubAssign<CoarseDuration> for CoarseInstant {
    fn sub_assign(&mut self, other: CoarseDuration) {
        *self = *self - other;
    }
}

impl Sub<CoarseInstant> for CoarseInstant {
    type Output = CoarseDuration;

    fn sub(self, other: CoarseInstant) -> CoarseDuration {
        self.duration_since(other)
    }
}

impl fmt::Debug for CoarseInstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoarseInstant")
            .field("secs", &self.secs)
            .finish()
    }
}
