// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![allow(clippy::needless_doctest_main)]

use crate::*;

use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// A measurement of a monotonically nondecreasing clock.
/// Opaque and useful only with `Duration`.
///
/// Compared to the standard library's implementation, the internal
/// representation is fixed at 64 bits and no attempts are made to correct for
/// os and platform specific bugs which may cause time to appear to move
/// backwards.
///
/// Another unique feature is `Duration::recent()` may be used to return a
/// cached view of the underlying clock. `rustcommon_time::clock_refresh` should
/// be used periodically to refresh the cached view of the underlying clock.
/// This is particularly useful for when the cost of the system calls to read
/// the underlying clock are too high but instants with an error bounded by the
/// frequency of calls to refresh the clock are acceptable.
///
/// Note that instants are not guaranteed to be **steady**. In other words, each
/// tick of the underlying clock may not be the same length (e.g. some seconds
/// may be longer than others). An instant may jump forwards or experience time
/// dilation (slow down or speed up).
///
/// Instants are opaque types that can only be compared to one another. There is
/// no method to get "the number of seconds" from an instant. Instead, it only
/// allows measuring the duration between two instants (or comparing two
/// instants).
///
/// Example:
///
/// ```no_run
/// use rustcommon_time::{Duration, Instant};
/// use std::thread::sleep;
///
/// fn main() {
///    let now = Instant::now();
///
///    // we sleep for 2 seconds
///    sleep(core::time::Duration::new(2, 0));
///    // it prints '2'
///    println!("{}", now.elapsed().as_secs());
/// }
/// ```
///
/// # Underlying System calls
/// Currently, the following system calls are being used to get the current time using `now()`:
///
/// |  Platform |               System call                                            |
/// |:---------:|:--------------------------------------------------------------------:|
/// | UNIX      | [clock_gettime (Monotonic Clock)]                                    |
/// | Darwin    | [mach_absolute_time]                                                 |
/// | Windows   | [QueryPerformanceCounter]                                            |
///
/// [QueryPerformanceCounter]: https://docs.microsoft.com/en-us/windows/win32/api/profileapi/nf-profileapi-queryperformancecounter
/// [clock_gettime (Monotonic Clock)]: https://linux.die.net/man/3/clock_gettime
/// [mach_absolute_time]: https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/KernelProgramming/services/services.html
///
/// **Disclaimer:** These system calls might change over time.
///
/// > Note: mathematical operations like [`add`] may panic if the underlying
/// > structure cannot represent the new point in time.
///
/// [`add`]: Instant::add
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant {
    pub(crate) nanos: u64,
}

#[cfg(all(not(target_os = "macos"), not(target_os = "ios"), unix))]
fn now() -> Instant {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    unsafe {
        libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
    }
    Instant {
        nanos: ts.tv_sec as u64 * NANOS_PER_SEC + ts.tv_nsec as u64,
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn now() -> Instant {
    use mach::mach_time::{mach_absolute_time, mach_timebase_info};
    use std::sync::Once;
    unsafe {
        let time = mach_absolute_time();

        let info = {
            static mut INFO: mach_timebase_info = mach_timebase_info { numer: 0, denom: 0 };
            static ONCE: std::sync::Once = Once::new();

            ONCE.call_once(|| {
                mach_timebase_info(&mut INFO);
            });
            &INFO
        };
        Instant {
            nanos: time * info.numer as u64 / info.denom as u64,
        }
    }
}

#[cfg(target_os = "windows")]
fn now() -> Instant {
    use std::mem;
    use winapi::um::profileapi;
    use winapi::um::winnt::LARGE_INTEGER;
    lazy_static! {
        static ref PRF_FREQUENCY: u64 = {
            unsafe {
                let mut frq = mem::uninitialized();
                let res = profileapi::QueryPerformanceFrequency(&mut frq);
                debug_assert_ne!(res, 0, "Failed to query performance frequency, {}", res);
                let frq = *frq.QuadPart() as u64;
                frq
            }
        };
    }
    let cnt = unsafe {
        let mut cnt = mem::uninitialized();
        debug_assert_eq!(mem::align_of::<LARGE_INTEGER>(), 8);
        let res = profileapi::QueryPerformanceCounter(&mut cnt);
        debug_assert_ne!(res, 0, "Failed to query performance counter {}", res);
        *cnt.QuadPart() as u64
    };

    Instant {
        nanos: (cnt as f64 / (*PRF_FREQUENCY as f64 / (NANOS_PER_SEC as f64))) as u64,
    }
}

impl Instant {
    /// Returns an instant corresponding to "now".
    ///
    /// # Examples
    ///
    /// ```
    /// use rustcommon_time::Instant;
    ///
    /// let now = Instant::now();
    /// ```
    pub fn now() -> Instant {
        now()
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
    /// use rustcommon_time::Instant;
    ///
    /// let recent = Instant::recent();
    /// ```
    pub fn recent() -> Instant {
        CLOCK.recent_precise()
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
    /// use rustcommon_time::Instant;
    /// use std::thread::sleep;
    ///
    /// let now = Instant::now();
    /// sleep(core::time::Duration::new(1, 0));
    /// let new_now = Instant::now();
    /// println!("{:?}", new_now.duration_since(now));
    /// ```
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        let nanos = self
            .nanos
            .checked_sub(earlier.nanos)
            .expect("supplied instant is later than self");
        Duration { nanos }
    }

    /// Returns the amount of time elapsed from another instant to this one,
    /// or None if that instant is later than this one.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rustcommon_time::Instant;
    /// use std::thread::sleep;
    ///
    /// let now = Instant::now();
    /// sleep(core::time::Duration::new(1, 0));
    /// let new_now = Instant::now();
    /// println!("{:?}", new_now.checked_duration_since(now));
    /// println!("{:?}", now.checked_duration_since(new_now)); // None
    /// ```
    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        let nanos = self.nanos.checked_sub(earlier.nanos)?;
        Some(Duration { nanos })
    }

    /// Returns the amount of time elapsed from another instant to this one,
    /// or zero duration if that instant is later than this one.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rustcommon_time::Instant;
    /// use std::thread::sleep;
    ///
    /// let now = Instant::now();
    /// sleep(core::time::Duration::new(1, 0));
    /// let new_now = Instant::now();
    /// println!("{:?}", new_now.saturating_duration_since(now));
    /// println!("{:?}", now.saturating_duration_since(new_now)); // 0ns
    /// ```
    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        let nanos = self.nanos.saturating_sub(earlier.nanos);
        Duration { nanos }
    }

    /// Returns the amount of time elapsed since this instant was created.
    ///
    /// # Panics
    ///
    /// This function may panic if the current time is earlier than this
    /// instant, which is something that can happen if an `Instant` is
    /// produced synthetically.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::thread::sleep;
    /// use rustcommon_time::{Duration, Instant};
    ///
    /// let instant = Instant::now();
    /// sleep(core::time::Duration::from_secs(3));
    /// assert!(instant.elapsed() >= Duration::from_secs(3));
    /// ```
    pub fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }

    /// Returns `Some(t)` where `t` is the time `self + duration` if `t` can be represented as
    /// `Instant` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
        let nanos = self.nanos.checked_add(duration.nanos)?;
        Some(Instant { nanos })
    }

    /// Returns `Some(t)` where `t` is the time `self - duration` if `t` can be represented as
    /// `Instant` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    pub fn checked_sub(&self, duration: Duration) -> Option<Instant> {
        let nanos = self.nanos.checked_sub(duration.nanos)?;
        Some(Instant { nanos })
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    /// # Panics
    ///
    /// This function may panic if the resulting point in time cannot be represented by the
    /// underlying data structure. See [`Instant::checked_add`] for a version without panic.
    fn add(self, other: Duration) -> Instant {
        self.checked_add(other)
            .expect("overflow when adding duration to instant")
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, other: Duration) {
        *self = *self + other;
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, other: Duration) -> Instant {
        self.checked_sub(other)
            .expect("overflow when subtracting duration from instant")
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, other: Duration) {
        *self = *self - other;
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, other: Instant) -> Duration {
        self.duration_since(other)
    }
}

impl fmt::Debug for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Instant")
            .field("nanos", &self.nanos)
            .finish()
    }
}
