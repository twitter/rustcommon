// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::sync::atomic::AtomicUsize;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

mod datetime;
mod duration;
mod instant;
#[macro_use]
mod macros;
mod units;
mod unix;

pub use datetime::*;
pub use duration::*;
pub use instant::*;
pub use units::*;
pub use unix::*;

pub(crate) const NANOS_PER_SEC: u64 = 1_000_000_000;

const UNINITIALIZED: usize = 0;
const INITIALIZED: usize = 1;
const REFRESHING: usize = 2;

// We initialize the clock for the static lifetime.
static CLOCK: Clock = Clock::new();

/// Update the cached view of the clock by reading the underlying clock.
pub fn refresh_clock() {
    CLOCK.refresh()
}

// Clock provides functionality to get current and recent times
struct Clock {
    state: AtomicUsize,
    coarse: Instant<Seconds<AtomicU32>>,
    precise: Instant<Nanoseconds<AtomicU64>>,
    coarse_unix: UnixInstant<Seconds<AtomicU32>>,
    precise_unix: UnixInstant<Nanoseconds<AtomicU64>>,
}

impl Clock {
    const fn new() -> Self {
        Clock {
            // holds the clock state, start as an uninitialized clock
            state: AtomicUsize::new(UNINITIALIZED),

            // store a monotonic clock reading
            coarse: Instant {
                inner: Seconds {
                    inner: AtomicU32::new(0),
                },
            },
            precise: Instant {
                inner: Nanoseconds {
                    inner: AtomicU64::new(0),
                },
            },

            // store a monotonic clock reading
            coarse_unix: UnixInstant {
                inner: Seconds {
                    inner: AtomicU32::new(0),
                },
            },
            precise_unix: UnixInstant {
                inner: Nanoseconds {
                    inner: AtomicU64::new(0),
                },
            },
        }
    }

    fn initialize(&self) {
        if self.state.load(Ordering::Relaxed) == UNINITIALIZED {
            self.refresh();
        }
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
                        Ordering::Acquire,
                        Ordering::Acquire,
                    )
                    .is_ok()
                {
                    let mut ts = libc::timespec {
                        tv_sec: 0,
                        tv_nsec: 0,
                    };
                    unsafe {
                        libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
                    }
                    self.coarse.store(
                        Instant {
                            inner: Seconds::from(ts),
                        },
                        Ordering::Release,
                    );
                    self.precise.store(
                        Instant {
                            inner: Nanoseconds::from(ts),
                        },
                        Ordering::Release,
                    );

                    let mut ts = libc::timespec {
                        tv_sec: 0,
                        tv_nsec: 0,
                    };
                    unsafe {
                        libc::clock_gettime(libc::CLOCK_REALTIME, &mut ts);
                    }
                    self.coarse_unix.store(
                        UnixInstant {
                            inner: Seconds::from(ts),
                        },
                        Ordering::Release,
                    );
                    self.precise_unix.store(
                        UnixInstant {
                            inner: Nanoseconds::from(ts),
                        },
                        Ordering::Release,
                    );

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
                        Ordering::Acquire,
                        Ordering::Acquire,
                    )
                    .is_ok()
                {
                    let mut ts = libc::timespec {
                        tv_sec: 0,
                        tv_nsec: 0,
                    };
                    unsafe {
                        libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
                    }

                    let now: Instant<Nanoseconds<u64>> = Instant {
                        inner: Nanoseconds::from(ts),
                    };

                    let previous = self.precise.load(Ordering::Acquire);

                    // this makes sure we're truly monotonic even if there are
                    // platform bugs
                    if now > previous {
                        self.precise.store(now, Ordering::Release);
                        self.coarse.store(
                            Instant {
                                inner: Seconds::from(ts),
                            },
                            Ordering::Release,
                        );
                    }

                    // update unix time
                    let mut ts = libc::timespec {
                        tv_sec: 0,
                        tv_nsec: 0,
                    };
                    unsafe {
                        libc::clock_gettime(libc::CLOCK_REALTIME, &mut ts);
                    }

                    // unconditionally set unix time, which may move backwards
                    self.coarse_unix.store(
                        UnixInstant {
                            inner: Seconds::from(ts),
                        },
                        Ordering::Release,
                    );
                    self.precise_unix.store(
                        UnixInstant {
                            inner: Nanoseconds::from(ts),
                        },
                        Ordering::Release,
                    );

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

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn basic() {
        let now = Instant::<Nanoseconds<u64>>::now();
        std::thread::sleep(std::time::Duration::new(1, 0));
        let elapsed = now.elapsed();
        assert!(elapsed.as_secs_f64() >= 1.0);
        assert!(elapsed.as_secs() >= 1);
        assert!(elapsed.as_nanos() >= NANOS_PER_SEC);

        let t0 = Instant::<Nanoseconds<u64>>::recent();
        std::thread::sleep(std::time::Duration::new(1, 0));
        assert_eq!(Instant::<Nanoseconds<u64>>::recent(), t0);
        refresh_clock();
        let t1 = Instant::<Nanoseconds<u64>>::recent();
        assert!((t1 - t0).as_secs_f64() >= 1.0);
        assert!((t1 - t0).as_secs() >= 1);
    }
}
