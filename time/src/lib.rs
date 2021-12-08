// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::sync::atomic::AtomicUsize;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use time::OffsetDateTime;

pub use std::time::SystemTime;

mod datetime;
mod duration;
mod instant;

pub use datetime::*;
pub use duration::*;
pub use instant::*;

const MILLIS_PER_SEC: u64 = 1_000;
const MICROS_PER_SEC: u64 = 1_000_000;
const NANOS_PER_SEC: u64 = 1_000_000_000;
const NANOS_PER_MILLI: u64 = 1_000_000;
const NANOS_PER_MICRO: u64 = 1_000;

const UNINITIALIZED: usize = 0;
const INITIALIZED: usize = 1;
const REFRESHING: usize = 2;

// We initialize the clock for the static lifetime.
static CLOCK: Clock = Clock::new();

// convenience functions

/// Refresh the clock and return the current instant with high precision.
pub fn now_precise() -> Instant {
    CLOCK.refresh();
    CLOCK.recent_precise()
}

/// Refresh the clock and return the current instant with reduced precision.
pub fn now_coarse() -> CoarseInstant {
    CLOCK.refresh();
    CLOCK.recent_coarse()
}

/// Refresh the clock and return the current system time.
pub fn now_system() -> SystemTime {
    CLOCK.refresh();
    CLOCK.recent_system()
}

/// Refresh the clock and return the current unix time in seconds.
pub fn now_unix() -> u32 {
    CLOCK.refresh();
    CLOCK.recent_unix()
}

/// Refresh the clock and return the current `DateTime` in the UTC timezone.
pub fn now_utc() -> DateTime {
    CLOCK.refresh();
    CLOCK.recent_utc()
}

/// Returns a recent precise instant by reading a cached view of the clock.
pub fn recent_precise() -> Instant {
    CLOCK.recent_precise()
}

/// Returns a recent coarse instant by reading a cached view of the clock.
pub fn recent_coarse() -> CoarseInstant {
    CLOCK.recent_coarse()
}

/// Returns the system time by reaching a cached view of the clock.
pub fn recent_system() -> SystemTime {
    CLOCK.recent_system()
}

/// Returns the unix time in seconds by reading a cached view of the clock.
pub fn recent_unix() -> u32 {
    CLOCK.recent_unix()
}

/// Returns the unix time in nanoseconds by reading a cached view of the clock.
pub fn recent_unix_precise() -> u64 {
    CLOCK.recent_unix_precise()
}

/// Returns a `DateTime` in UTC from a cached view of the clock.
pub fn recent_utc() -> DateTime {
    CLOCK.recent_utc()
}

/// Update the cached view of the clock by reading the underlying clock.
pub fn refresh_clock() {
    CLOCK.refresh()
}

// Clock provides functionality to get current and recent times
struct Clock {
    state: AtomicUsize,
    recent_coarse: AtomicCoarseInstant,
    recent_precise: AtomicInstant,
    recent_unix: AtomicU64,
}

impl Clock {
    fn initialize(&self) {
        if self.state.load(Ordering::Relaxed) == UNINITIALIZED {
            self.refresh();
        }
    }

    /// Return a cached precise time
    fn recent_precise(&self) -> Instant {
        self.initialize();
        self.recent_precise.load(Ordering::Relaxed)
    }

    /// Return a cached coarse time
    fn recent_coarse(&self) -> CoarseInstant {
        self.initialize();
        self.recent_coarse.load(Ordering::Relaxed)
    }

    /// Return a cached SystemTime
    fn recent_system(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(self.recent_unix().into())
    }

    /// Return a cached UNIX time in seconds
    fn recent_unix(&self) -> u32 {
        self.initialize();
        (self.recent_unix.load(Ordering::Relaxed) / NANOS_PER_SEC) as u32
    }

    /// Return a cached UNIX time in nanoseconds
    fn recent_unix_precise(&self) -> u64 {
        self.initialize();
        self.recent_unix.load(Ordering::Relaxed)
    }

    /// Return a cached UTC DateTime
    fn recent_utc(&self) -> DateTime {
        // This unwrap is safe, because we use a ~35bits to hold seconds. Tests
        // enforce the correctness of this below.
        let now = self.recent_unix_precise();
        let seconds = now / NANOS_PER_SEC;
        let nanos = now % NANOS_PER_SEC;
        let recent = OffsetDateTime::from_unix_timestamp(seconds as i64).unwrap()
            + time::Duration::nanoseconds(nanos as i64);
        DateTime { inner: recent }
    }

    /// Refresh the cached time
    fn refresh(&self) {
        match self.state.load(Ordering::Relaxed) {
            UNINITIALIZED => {
                if self
                    .state
                    .compare_exchange(
                        UNINITIALIZED,
                        REFRESHING,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    // get the current precise time and cache it
                    let precise = Instant::now();
                    self.recent_precise.store(precise, Ordering::Release);

                    // set the coarse time by converting the precise time
                    let coarse = CoarseInstant {
                        secs: (precise.nanos / NANOS_PER_SEC) as u32,
                    };
                    self.recent_coarse.store(coarse, Ordering::Release);

                    // get the current unix time from the system and store it
                    let unix = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos() as u64;
                    self.recent_unix.store(unix, Ordering::Release);

                    // finalize initialization
                    self.state.store(INITIALIZED, Ordering::Release);
                }
                // if we raced, we should block until the other thread completes
                // initialization
                while self.state.load(Ordering::Relaxed) != INITIALIZED {}
            }
            INITIALIZED => {
                if self
                    .state
                    .compare_exchange(
                        INITIALIZED,
                        REFRESHING,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    // get the current precise time
                    let precise = Instant::now();

                    // increment unix time by elapsed time in nanoseconds between
                    // refreshes
                    self.recent_unix.fetch_add(
                        (precise - recent_precise()).as_nanos() as u64,
                        Ordering::Relaxed,
                    );

                    // set coarse time to precise time converted to seconds
                    let coarse = CoarseInstant {
                        secs: (precise.nanos / NANOS_PER_SEC) as u32,
                    };
                    self.recent_coarse.store(coarse, Ordering::Relaxed);

                    // set precise time
                    self.recent_precise.store(precise, Ordering::Release);

                    // finalize refresh
                    self.state.store(INITIALIZED, Ordering::Relaxed);
                }
                // if we raced, we should block until the other thread completes
                // initialization
                while self.state.load(Ordering::Relaxed) != INITIALIZED {}
            }
            REFRESHING => {
                // if we raced, we should block until the other thread completes
                // initialization
                while self.state.load(Ordering::Relaxed) != INITIALIZED {}
            }
            _ => {
                unreachable!()
            }
        }
    }
}

impl Clock {
    const fn new() -> Self {
        Clock {
            state: AtomicUsize::new(UNINITIALIZED),
            recent_coarse: AtomicCoarseInstant {
                secs: AtomicU32::new(0),
            },
            recent_precise: AtomicInstant {
                nanos: AtomicU64::new(0),
            },
            recent_unix: AtomicU64::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    // This tests the direct interface to `Instant` and `Duration`
    fn basic() {
        let now = Instant::now();
        std::thread::sleep(std::time::Duration::new(1, 0));
        let elapsed = now.elapsed();
        assert!(elapsed.as_secs_f64() >= 1.0);
        assert!(elapsed.as_secs() >= 1);
        assert!(elapsed.as_nanos() >= NANOS_PER_SEC.into());
    }

    #[test]
    /// This tests the system time handling
    fn system() {
        let recent = recent_system();
        let now = std::time::SystemTime::now();
        assert!((now.duration_since(recent).unwrap()).as_secs() <= 1);
    }

    #[test]
    // This tests the 'clock' interface which is hidden behind macros
    fn clock() {
        let now = Instant::now();
        std::thread::sleep(std::time::Duration::new(1, 0));
        let elapsed = now.elapsed();
        assert!(elapsed.as_secs() >= 1);
        assert!(elapsed.as_nanos() >= NANOS_PER_SEC.into());

        let t0 = Instant::recent();
        let t0_c = Instant::recent();
        std::thread::sleep(std::time::Duration::new(1, 0));
        assert_eq!(Instant::recent(), t0);
        refresh_clock();
        let t1 = Instant::recent();
        let t1_c = Instant::recent();
        assert!((t1 - t0).as_secs_f64() >= 1.0);
        assert!((t1_c - t0_c).as_secs() >= 1);
    }

    #[test]
    fn cached_time() {
        let previous_precise = recent_precise();
        let previous_unix = recent_unix_precise();
        std::thread::sleep(std::time::Duration::from_millis(50));
        refresh_clock();
        let current_precise = recent_precise();
        let current_unix = recent_unix_precise();

        assert_eq!(
            (current_precise - previous_precise).as_nanos() as u64,
            current_unix - previous_unix
        );
        assert!(current_unix - previous_unix > 50_000_000);
        assert!(current_unix - previous_unix < 100_000_000);
    }
}
