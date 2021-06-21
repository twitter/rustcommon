// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;
use core::sync::atomic::AtomicU32;

/// An `AtomicCoarseDuration` is an atomic equivalent of [`CoarseDuration`].
///
/// # Examples
///
/// ```
/// use rustcommon_time::{AtomicCoarseDuration, CoarseDuration};
/// use core::sync::atomic::Ordering;
///
/// let one_second = AtomicCoarseDuration::new(1);
/// assert_eq!(one_second.load(Ordering::Relaxed).as_secs(), 1);
///
/// let ten_seconds = AtomicCoarseDuration::from_secs(10);
/// ```
pub struct AtomicCoarseDuration {
    secs: AtomicU32,
}

impl AtomicCoarseDuration {
    /// Creates a new `Duration` from the specified number of whole seconds and
    /// additional nanoseconds.
    ///
    /// If the number of nanoseconds is greater than 1 billion (the number of
    /// nanoseconds in a second), then it will carry over into the seconds
    /// provided.
    ///
    /// # Panics
    ///
    /// This constructor will panic if the carry from the nanoseconds overflows
    /// the seconds counter.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    ///
    /// let five_seconds = Duration::new(5, 0);
    /// ```
    #[inline]
    pub fn new(secs: u32) -> Self {
        Self {
            secs: AtomicU32::new(secs),
        }
    }

    /// Creates a new `AtomicDuration` from the specified number of whole seconds.
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
    pub const fn from_secs(secs: u32) -> Self {
        Self {
            secs: AtomicU32::new(secs),
        }
    }

    /// Consumes the atomic and returns the contained value.
    ///
    /// This is safe because passing `self` by value guarantees that no other
    /// threads are concurrently accessing the atomic data.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::{AtomicCoarseDuration, CoarseDuration};
    ///
    /// let duration = AtomicCoarseDuration::from_secs(1337);
    /// assert_eq!(duration.into_inner(), CoarseDuration::from_secs(1337));
    /// ```
    pub fn into_inner(self) -> CoarseDuration {
        CoarseDuration {
            secs: self.secs.into_inner(),
        }
    }

    /// Loads the duration from the atomic duration.
    ///
    /// `load` takes an `Ordering` argument which describes the memory ordering
    /// of this operation. Possible values are `SeqCst`, `Acquire`, and
    /// `Relaxed`.
    ///
    /// # Panics
    /// Panics if `order` is `Release` or `AcqRel`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::{AtomicCoarseDuration, CoarseDuration};
    /// use core::sync::atomic::Ordering;
    ///
    /// let duration = AtomicCoarseDuration::from_secs(5);
    /// assert_eq!(duration.load(Ordering::Relaxed), CoarseDuration::from_secs(5));
    /// ```
    pub fn load(&self, order: Ordering) -> CoarseDuration {
        CoarseDuration {
            secs: self.secs.load(order),
        }
    }

    /// Stores the duration from the atomic duration.
    ///
    /// `store` takes an `Ordering` argument which describes the memory ordering
    /// of this operation. Possible values are `SeqCst`, `Release`, and
    /// `Relaxed`.
    ///
    /// # Panics
    /// Panics if `order` is `Acquire` or `AcqRel`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::{AtomicCoarseDuration, CoarseDuration};
    /// use core::sync::atomic::Ordering;
    ///
    /// let duration = AtomicCoarseDuration::from_secs(5);
    ///
    /// duration.store(CoarseDuration::from_secs(10), Ordering::Relaxed);
    /// assert_eq!(duration.load(Ordering::Relaxed), CoarseDuration::from_secs(10));
    /// ```
    pub fn store(&self, val: CoarseDuration, order: Ordering) {
        self.secs.store(val.secs, order)
    }

    /// Stores the duration from the atomic duration, returning the previous
    /// value.
    ///
    /// `swap` takes an `Ordering` argument which describes the memory ordering
    /// of this operation. All ordering modes are possible. Note that using
    /// `Acquire` makes the store part of this operation `Relaxed`, and using
    /// `Release` makes the load part `Relaxed`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::{AtomicCoarseDuration, CoarseDuration};
    /// use core::sync::atomic::Ordering;
    ///
    /// let duration = AtomicCoarseDuration::from_secs(5);
    ///
    /// assert_eq!(duration.swap(CoarseDuration::from_secs(10), Ordering::Relaxed), CoarseDuration::from_secs(5));
    /// ```
    pub fn swap(&self, val: CoarseDuration, order: Ordering) -> CoarseDuration {
        CoarseDuration {
            secs: self.secs.swap(val.secs, order),
        }
    }

    /// Stores the duration from the atomic duration if the current value is the
    /// same as the `current` value.
    ///
    /// The return value is a result indicating whether the new value was
    /// written and containing the previous value. On success this value is
    /// guaranteed to be equal to current.
    ///
    /// `compare_exchange` takes two `Ordering` arguments to describe the memory
    /// ordering of this operation. `success` describes the required ordering
    /// for the read-modify-write operation that takes place if the comparison
    /// with `current` succeeds. `failure` describes the required ordering for
    /// the load operation that takes place when the comparison fails. Using
    /// `Acquire` as success ordering makes the store part of this operation
    /// `Relaxed`, and using `Release makes the successful load `Relaxed`. The
    /// failure ordering can only be, `SeqCst`, `Acquire`, or `Relaxed` and must
    /// be equivalent to or weaker than the success ordering.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::{AtomicCoarseDuration, CoarseDuration};
    /// use core::sync::atomic::Ordering;
    ///
    /// let duration = AtomicCoarseDuration::from_secs(5);
    ///
    /// assert_eq!(duration.compare_exchange(
    ///     CoarseDuration::from_secs(5),
    ///     CoarseDuration::from_secs(10),
    ///     Ordering::Acquire,
    ///     Ordering::Relaxed),
    ///     Ok(CoarseDuration::from_secs(5)));
    /// assert_eq!(duration.load(Ordering::Relaxed), CoarseDuration::from_secs(10));
    ///
    /// assert_eq!(duration.compare_exchange(
    ///     CoarseDuration::from_secs(6),
    ///     CoarseDuration::from_secs(12),
    ///     Ordering::Acquire,
    ///     Ordering::Relaxed),
    ///     Err(CoarseDuration::from_secs(10)));
    /// assert_eq!(duration.load(Ordering::Relaxed), CoarseDuration::from_secs(10));
    ///
    /// ```
    pub fn compare_exchange(
        &self,
        current: CoarseDuration,
        new: CoarseDuration,
        success: Ordering,
        failure: Ordering,
    ) -> Result<CoarseDuration, CoarseDuration> {
        match self
            .secs
            .compare_exchange(current.secs, new.secs, success, failure)
        {
            Ok(secs) => Ok(CoarseDuration { secs }),
            Err(secs) => Err(CoarseDuration { secs }),
        }
    }

    /// Stores the duration from the atomic duration if the current value is the
    /// same as the `current` value.
    ///
    /// Unlike `AtomicCoarseDuration::compare_exchange`, this function is
    /// allowed to spuriously fail even when the comparison succeeds, which can
    /// result in more efficient code on some platforms. The return value is a
    /// result indicating whether the new value was written and containing the
    /// previous value.
    ///
    /// `compare_exchange_weak` takes two `Ordering` arguments to describe the
    /// memory ordering of this operation. `success` describes the required
    /// ordering for the read-modify-write operation that takes place if the
    /// comparison with `current` succeeds. `failure` describes the required
    /// ordering for the load operation that takes place when the comparison
    /// fails. Using `Acquire` as success ordering makes the store part of this
    /// operation `Relaxed`, and using `Release makes the successful load
    /// `Relaxed`. The failure ordering can only be, `SeqCst`, `Acquire`, or
    /// `Relaxed` and must be equivalent to or weaker than the success ordering.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::{AtomicCoarseDuration, CoarseDuration};
    /// use core::sync::atomic::Ordering;
    ///
    /// let duration = AtomicCoarseDuration::from_secs(5);
    ///
    /// let mut old = duration.load(Ordering::Relaxed);
    /// loop {
    ///     let new = old.saturating_mul(2);
    ///     match duration.compare_exchange_weak(old, new, Ordering::SeqCst, Ordering::Relaxed) {
    ///         Ok(_) => break,
    ///         Err(x) => old = x,
    ///     }
    /// }
    /// ```
    pub fn compare_exchange_weak(
        &self,
        current: CoarseDuration,
        new: CoarseDuration,
        success: Ordering,
        failure: Ordering,
    ) -> Result<CoarseDuration, CoarseDuration> {
        match self
            .secs
            .compare_exchange_weak(current.secs, new.secs, success, failure)
        {
            Ok(secs) => Ok(CoarseDuration { secs }),
            Err(secs) => Err(CoarseDuration { secs }),
        }
    }
}
