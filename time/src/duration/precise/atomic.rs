use crate::*;

/// An `AtomicDuration` is an atomic equivalent of [`Duration`].
///
/// # Examples
///
/// ```
/// use rustcommon_time::AtomicDuration;
/// use core::sync::atomic::Ordering;
///
/// let duration = AtomicDuration::new(5, 5);
/// assert_eq!(duration.load(Ordering::Relaxed).as_secs(), 5);
/// assert_eq!(duration.load(Ordering::Relaxed).subsec_nanos(), 5);
///
/// let ten_millis = AtomicDuration::from_millis(10);
/// ```
pub struct AtomicDuration {
    nanos: AtomicU64,
}

impl AtomicDuration {
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
    /// use core::sync::atomic::Ordering;
    ///
    /// let five_seconds = Duration::new(5, 0);
    /// ```
    #[inline]
    pub fn new(secs: u64, nanos: u32) -> Self {
        let secs_ns = secs
            .checked_mul(NANOS_PER_SEC)
            .expect("number of seconds caused overflow");
        let nanos = secs_ns
            .checked_add(nanos as u64)
            .expect("total duration caused overflow");
        Self {
            nanos: AtomicU64::new(nanos),
        }
    }

    /// Creates a new `AtomicDuration` from the specified number of whole seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Duration;
    /// use core::sync::atomic::Ordering;
    ///
    /// let duration = Duration::from_secs(5);
    ///
    /// assert_eq!(5, duration.as_secs());
    /// assert_eq!(0, duration.subsec_nanos());
    /// ```
    #[inline]
    pub fn from_secs(secs: u64) -> Self {
        let nanos = secs
            .checked_mul(NANOS_PER_SEC)
            .expect("total duration caused overflow");
        Self {
            nanos: AtomicU64::new(nanos),
        }
    }

    /// Creates a new `AtomicDuration` from the specified number of milliseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::AtomicDuration;
    /// use core::sync::atomic::Ordering;
    ///
    /// let duration = AtomicDuration::from_millis(2569);
    ///
    /// assert_eq!(2, duration.load(Ordering::Relaxed).as_secs());
    /// assert_eq!(569_000_000, duration.load(Ordering::Relaxed).subsec_nanos());
    /// ```
    #[inline]
    pub fn from_millis(millis: u64) -> Self {
        let nanos = millis
            .checked_mul(NANOS_PER_MILLI)
            .expect("total duration caused overflow");
        Self {
            nanos: AtomicU64::new(nanos),
        }
    }

    /// Creates a new `AtomicDuration` from the specified number of microseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::AtomicDuration;
    /// use core::sync::atomic::Ordering;
    ///
    /// let duration = AtomicDuration::from_micros(1_000_002);
    ///
    /// assert_eq!(1, duration.load(Ordering::Relaxed).as_secs());
    /// assert_eq!(2000, duration.load(Ordering::Relaxed).subsec_nanos());
    /// ```
    #[inline]
    pub fn from_micros(micros: u64) -> Self {
        let nanos = micros
            .checked_mul(NANOS_PER_MICRO)
            .expect("total duration caused overflow");
        Self {
            nanos: AtomicU64::new(nanos),
        }
    }

    /// Creates a new `AtomicDuration` from the specified number of nanoseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::AtomicDuration;
    /// use core::sync::atomic::Ordering;
    ///
    /// let duration = AtomicDuration::from_nanos(1_000_000_123);
    ///
    /// assert_eq!(1, duration.load(Ordering::Relaxed).as_secs());
    /// assert_eq!(123, duration.load(Ordering::Relaxed).subsec_nanos());
    /// ```
    #[inline]
    pub const fn from_nanos(nanos: u64) -> Self {
        Self {
            nanos: AtomicU64::new(nanos),
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
    /// use rustcommon_time::{AtomicDuration, Duration};
    ///
    /// let duration = AtomicDuration::from_nanos(1337);
    /// assert_eq!(duration.into_inner(), Duration::from_nanos(1337));
    /// ```
    pub fn into_inner(self) -> Duration {
        Duration {
            nanos: self.nanos.into_inner(),
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
    /// use rustcommon_time::{AtomicDuration, Duration};
    /// use core::sync::atomic::Ordering;
    ///
    /// let duration = AtomicDuration::from_secs(5);
    /// assert_eq!(duration.load(Ordering::Relaxed), Duration::from_secs(5));
    /// ```
    pub fn load(&self, order: Ordering) -> Duration {
        Duration {
            nanos: self.nanos.load(order),
        }
    }

    /// Stores the duration into the atomic duration.
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
    /// use rustcommon_time::{AtomicDuration, Duration};
    /// use core::sync::atomic::Ordering;
    ///
    /// let duration = AtomicDuration::from_secs(5);
    ///
    /// duration.store(Duration::from_secs(10), Ordering::Relaxed);
    /// assert_eq!(duration.load(Ordering::Relaxed), Duration::from_secs(10));
    /// ```
    pub fn store(&self, val: Duration, order: Ordering) {
        self.nanos.store(val.nanos, order)
    }

    /// Stores the duration into the atomic duration, returning the previous
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
    /// use rustcommon_time::{AtomicDuration, Duration};
    /// use core::sync::atomic::Ordering;
    ///
    /// let duration = AtomicDuration::from_secs(5);
    ///
    /// assert_eq!(duration.swap(Duration::from_secs(10), Ordering::Relaxed), Duration::from_secs(5));
    /// ```
    pub fn swap(&self, val: Duration, order: Ordering) -> Duration {
        Duration {
            nanos: self.nanos.swap(val.nanos, order),
        }
    }

    /// Stores the duration into the atomic duration if the current value is the
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
    /// use rustcommon_time::{AtomicDuration, Duration};
    /// use core::sync::atomic::Ordering;
    ///
    /// let duration = AtomicDuration::from_secs(5);
    ///
    /// assert_eq!(duration.compare_exchange(
    ///     Duration::from_secs(5),
    ///     Duration::from_secs(10),
    ///     Ordering::Acquire,
    ///     Ordering::Relaxed),
    ///     Ok(Duration::from_secs(5)));
    /// assert_eq!(duration.load(Ordering::Relaxed), Duration::from_secs(10));
    ///
    /// assert_eq!(duration.compare_exchange(
    ///     Duration::from_secs(6),
    ///     Duration::from_secs(12),
    ///     Ordering::Acquire,
    ///     Ordering::Relaxed),
    ///     Err(Duration::from_secs(10)));
    /// assert_eq!(duration.load(Ordering::Relaxed), Duration::from_secs(10));
    ///
    /// ```
    pub fn compare_exchange(
        &self,
        current: Duration,
        new: Duration,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Duration, Duration> {
        match self
            .nanos
            .compare_exchange(current.nanos, new.nanos, success, failure)
        {
            Ok(nanos) => Ok(Duration { nanos }),
            Err(nanos) => Err(Duration { nanos }),
        }
    }

    /// Stores the duration into the atomic duration if the current value is the
    /// same as the `current` value.
    ///
    /// Unlike `AtomicDuration::compare_exchange`, this function is allowed to
    /// spuriously fail even when the comparison succeeds, which can result in
    /// more efficient code on some platforms. The return value is a result
    /// indicating whether the new value was written and containing the previous
    /// value.
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
    /// use rustcommon_time::{AtomicDuration, Duration};
    /// use core::sync::atomic::Ordering;
    ///
    /// let duration = AtomicDuration::from_secs(5);
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
        current: Duration,
        new: Duration,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Duration, Duration> {
        match self
            .nanos
            .compare_exchange_weak(current.nanos, new.nanos, success, failure)
        {
            Ok(nanos) => Ok(Duration { nanos }),
            Err(nanos) => Err(Duration { nanos }),
        }
    }
}
