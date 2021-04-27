use core::sync::atomic::AtomicBool;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;
use lazy_static::lazy_static;

mod duration;
mod instant;

pub use duration::*;
pub use instant::*;

const MILLIS_PER_SEC: u64 = 1_000;
const MICROS_PER_SEC: u64 = 1_000_000;
const NANOS_PER_SEC: u64 = 1_000_000_000;
const NANOS_PER_MILLI: u64 = 1_000_000;
const NANOS_PER_MICRO: u64 = 1_000;

// We initialize the clock for the static lifetime.
lazy_static! {
    static ref CLOCK: Clock = Clock::new();
}

fn _clock() -> &'static Clock {
    &CLOCK
}

// convenience functions

/// Returns a precise instant by reading the underlying clock.
pub fn now_precise() -> Instant {
    _clock().now_precise()
}

/// Returns a coarse instant by reading the underlying clock.
pub fn now_coarse() -> CoarseInstant {
    _clock().now_coarse()
}

/// Returns a recent precise instant by reading a cached view of the clock.
pub fn recent_precise() -> Instant {
    _clock().recent_precise()
}

/// Returns a recent coarse instant by reading a cached view of the clock.
pub fn recent_coarse() -> CoarseInstant {
    _clock().recent_coarse()
}

/// Update the cached view of the clock by reading the underlying clock.
pub fn refresh_clock() {
    _clock().refresh()
}

// Clock provides functionality to get current and recent times
struct Clock {
    recent_precise: AtomicInstant,
    recent_coarse: AtomicCoarseInstant,
    initialized: AtomicBool,
}

impl Clock {
    /// Return the current precise time
    fn now_precise(&self) -> Instant {
        Instant::now()
    }

    /// Return the current coarse time
    fn now_coarse(&self) -> CoarseInstant {
        CoarseInstant::now()
    }

    /// Return a cached precise time
    fn recent_precise(&self) -> Instant {
        if self.initialized.load(Ordering::Relaxed) {
            self.recent_precise.load(Ordering::Relaxed)
        } else {
            self.refresh();
            self.recent_precise.load(Ordering::Relaxed)
        }
    }

    /// Return a cached coarse time
    fn recent_coarse(&self) -> CoarseInstant {
        if self.initialized.load(Ordering::Relaxed) {
            self.recent_coarse.load(Ordering::Relaxed)
        } else {
            self.refresh();
            self.recent_coarse.load(Ordering::Relaxed)
        }
    }

    /// Refresh the cached time
    fn refresh(&self) {
        let precise = Instant::now();
        let coarse = CoarseInstant {
            secs: (precise.nanos / NANOS_PER_SEC) as u32,
        };
        self.recent_precise.store(precise, Ordering::Relaxed);
        self.recent_coarse.store(coarse, Ordering::Relaxed);
        self.initialized.store(true, Ordering::Relaxed);
    }
}

impl Clock {
    const fn new() -> Self {
        Clock {
            recent_precise: AtomicInstant {
                nanos: AtomicU64::new(0),
            },
            recent_coarse: AtomicCoarseInstant {
                secs: AtomicU32::new(0),
            },
            initialized: AtomicBool::new(false),
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
        assert!(elapsed.as_nanos() >= 1_000_000_000);
    }

    #[test]
    // This tests the 'clock' interface which is hidden behind macros
    fn clock() {
        let now = Instant::now();
        std::thread::sleep(std::time::Duration::new(1, 0));
        let elapsed = now.elapsed();
        assert!(elapsed.as_secs() >= 1);
        assert!(elapsed.as_nanos() >= 1_000_000_000);

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
}
