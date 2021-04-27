use crate::*;

use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;

/// An `AtomicDuration` is an atomic equivalent of [`Instant`].
///
/// # Examples
///
/// ```
/// use rustcommon_time::{AtomicInstant, Duration};
/// use core::sync::atomic::Ordering;
///
/// let now = AtomicInstant::now();
/// std::thread::sleep(core::time::Duration::from_secs(1));
/// assert!(now.load(Ordering::Relaxed).elapsed() >= Duration::from_secs(1));
/// ```
pub struct AtomicInstant {
    pub(crate) nanos: AtomicU64,
}

impl AtomicInstant {
    /// Returns an instant corresponding to "now".
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::AtomicInstant;
    ///
    /// let now = AtomicInstant::now();
    /// ```
    pub fn now() -> Self {
        let instant = Instant::now();
        Self {
            nanos: AtomicU64::new(instant.nanos),
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
    /// use rustcommon_time::AtomicInstant;
    ///
    /// let recent = AtomicInstant::recent();
    /// ```
    pub fn recent() -> Self {
        let instant = CLOCK.recent_precise();
        Self {
            nanos: AtomicU64::new(instant.nanos),
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
    /// use rustcommon_time::{AtomicInstant, Instant};
    ///
    /// let instant = AtomicInstant::now();
    /// std::thread::sleep(core::time::Duration::from_secs(1));
    /// assert!(instant.into_inner() < Instant::now());
    /// ```
    pub fn into_inner(self) -> Instant {
        Instant {
            nanos: self.nanos.into_inner(),
        }
    }

    /// Loads the instant from the atomic instant.
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
    /// use rustcommon_time::{AtomicInstant, Instant};
    /// use core::sync::atomic::Ordering;
    ///
    /// let instant = AtomicInstant::now();
    /// std::thread::sleep(core::time::Duration::from_secs(1));
    /// assert!(instant.load(Ordering::Relaxed) < Instant::now());
    /// ```
    pub fn load(&self, ordering: Ordering) -> Instant {
        Instant {
            nanos: self.nanos.load(ordering),
        }
    }

    /// Stores the instant into the atomic instant.
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
    /// use rustcommon_time::{AtomicInstant, Duration, Instant};
    /// use core::sync::atomic::Ordering;
    ///
    /// let instant = AtomicInstant::now();
    ///
    /// std::thread::sleep(core::time::Duration::from_secs(1));
    /// assert!(instant.load(Ordering::Relaxed).elapsed() >= Duration::from_secs(1));
    ///
    /// instant.store(Instant::now(), Ordering::Relaxed);
    /// assert!(instant.load(Ordering::Relaxed).elapsed() < Duration::from_secs(1));
    /// ```
    pub fn store(&self, value: Instant, ordering: Ordering) {
        self.nanos.store(value.nanos, ordering)
    }

    /// Stores the instant into the atomic instant, returning the previous
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
    /// use rustcommon_time::{AtomicInstant, Instant};
    /// use core::sync::atomic::Ordering;
    ///
    /// let now = AtomicInstant::now();
    /// std::thread::sleep(core::time::Duration::from_secs(1));
    /// let new_now = Instant::now();
    ///
    /// assert!(now.swap(new_now, Ordering::Relaxed) < new_now);
    /// ```
    pub fn swap(&self, val: Instant, order: Ordering) -> Instant {
        Instant {
            nanos: self.nanos.swap(val.nanos, order),
        }
    }

    /// Stores the instant into the atomic instant if the current value is the
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
    /// use rustcommon_time::{AtomicInstant, Duration, Instant};
    /// use core::sync::atomic::Ordering;
    ///
    /// let instant = AtomicInstant::now();
    /// let original = instant.load(Ordering::Relaxed);
    ///
    /// assert_eq!(instant.compare_exchange(
    ///     original,
    ///     original + Duration::from_secs(1),
    ///     Ordering::Acquire,
    ///     Ordering::Relaxed),
    ///     Ok(original));
    /// assert_eq!(instant.load(Ordering::Relaxed), original + Duration::from_secs(1));
    ///
    /// assert_eq!(instant.compare_exchange(
    ///     original,
    ///     Instant::now(),
    ///     Ordering::Acquire,
    ///     Ordering::Relaxed),
    ///     Err(original + Duration::from_secs(1)));
    /// assert_eq!(instant.load(Ordering::Relaxed), original + Duration::from_secs(1));
    ///
    /// ```
    pub fn compare_exchange(
        &self,
        current: Instant,
        new: Instant,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Instant, Instant> {
        match self
            .nanos
            .compare_exchange(current.nanos, new.nanos, success, failure)
        {
            Ok(nanos) => Ok(Instant { nanos }),
            Err(nanos) => Err(Instant { nanos }),
        }
    }

    /// Stores the instant into the atomic instant if the current value is the
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
    /// use rustcommon_time::{AtomicInstant, Instant};
    /// use core::sync::atomic::Ordering;
    ///
    /// let instant = AtomicInstant::now();
    ///
    /// let mut old = instant.load(Ordering::Relaxed);
    /// loop {
    ///     match instant.compare_exchange_weak(old, Instant::now(), Ordering::SeqCst, Ordering::Relaxed) {
    ///         Ok(_) => break,
    ///         Err(x) => old = x,
    ///     }
    /// }
    /// ```
    pub fn compare_exchange_weak(
        &self,
        current: Instant,
        new: Instant,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Instant, Instant> {
        match self
            .nanos
            .compare_exchange_weak(current.nanos, new.nanos, success, failure)
        {
            Ok(nanos) => Ok(Instant { nanos }),
            Err(nanos) => Err(Instant { nanos }),
        }
    }
}
