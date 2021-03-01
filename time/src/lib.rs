use core::sync::atomic::AtomicBool;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;

const NS_PER_MICROSECOND: u64 = 1_000;
const NS_PER_MILLISECOND: u64 = 1_000_000;
const NS_PER_SECOND: u64 = 1_000_000_000;

// We initialize the clock for the static lifetime.
// TODO(bmartin): this probably doesn't even need to be mutable...
static mut CLOCK: Clock = Clock::new();

fn _clock() -> &'static Clock {
    unsafe { &CLOCK }
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
    recent: AtomicInstant,
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
            self.recent.load(Ordering::Relaxed)
        } else {
            self.refresh();
            self.recent.load(Ordering::Relaxed)
        }
    }

    /// Return a cached coarse time
    fn recent_coarse(&self) -> CoarseInstant {
        CoarseInstant::from(self.recent_precise())
    }

    /// Refresh the cached time
    fn refresh(&self) {
        self.recent.store(Instant::now(), Ordering::Relaxed);
        self.initialized.store(true, Ordering::Relaxed);
    }
}

impl Clock {
    const fn new() -> Self {
        Clock {
            recent: AtomicInstant {
                ns: AtomicU64::new(0),
            },
            initialized: AtomicBool::new(false),
        }
    }
}

/// `Instant` is an opaque type that represents a moment in time.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Instant {
    ns: u64, // This is enough for 500 years without overflow
}

/// `CoarseInstant` is an opaque type that represents a moment in time to the
/// nearest second.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct CoarseInstant {
    s: u32, // This is enough for >100 years without overflow
}

impl From<Instant> for CoarseInstant {
    fn from(instant: Instant) -> Self {
        Self {
            s: (instant.ns / NS_PER_SECOND) as u32,
        }
    }
}

/// `AtomicCoarseInstant` is an opaque type that represents a moment in time to
/// the nearest second. Unlike `CoarseInstant`, it is thread-safe.
#[derive(Debug)]
pub struct AtomicCoarseInstant {
    s: AtomicU32, // This is enough for >100 years without overflow
}

/// `AtomicInstant` is an opaque type that represents a moment in time. Unlike
/// `Instant`, it is thread-safe.
pub struct AtomicInstant {
    ns: AtomicU64,
}

impl AtomicInstant {
    pub fn now() -> Self {
        let instant = Instant::now();
        Self {
            ns: AtomicU64::new(instant.ns),
        }
    }

    pub fn recent() -> Self {
        let instant = _clock().recent_precise();
        Self {
            ns: AtomicU64::new(instant.ns),
        }
    }

    pub fn load(&self, ordering: Ordering) -> Instant {
        Instant {
            ns: self.ns.load(ordering),
        }
    }

    pub fn store(&self, value: Instant, ordering: Ordering) {
        self.ns.store(value.ns, ordering)
    }

    pub fn fetch_add(&self, other: Duration, ordering: Ordering) -> Instant {
        Instant {
            ns: self.ns.fetch_add(other.ns, ordering),
        }
    }

    pub fn fetch_sub(&self, other: Duration, ordering: Ordering) -> Instant {
        Instant {
            ns: self.ns.fetch_sub(other.ns, ordering),
        }
    }

    pub fn refresh(&self, ordering: Ordering) {
        self.store(Instant::now(), ordering)
    }

    pub fn elapsed(&self, ordering: Ordering) -> Duration {
        self.load(ordering).elapsed()
    }
}

impl AtomicCoarseInstant {
    pub fn now() -> Self {
        let instant = CoarseInstant::now();
        Self {
            s: AtomicU32::new(instant.s),
        }
    }

    pub fn recent() -> Self {
        let instant = _clock().recent_coarse();
        Self {
            s: AtomicU32::new(instant.s),
        }
    }

    pub fn load(&self, ordering: Ordering) -> CoarseInstant {
        CoarseInstant {
            s: self.s.load(ordering),
        }
    }

    pub fn store(&self, value: CoarseInstant, ordering: Ordering) {
        self.s.store(value.s, ordering)
    }

    pub fn fetch_add(&self, other: CoarseDuration, ordering: Ordering) -> CoarseInstant {
        CoarseInstant {
            s: self.s.fetch_add(other.s, ordering),
        }
    }

    pub fn fetch_sub(&self, other: CoarseDuration, ordering: Ordering) -> CoarseInstant {
        CoarseInstant {
            s: self.s.fetch_sub(other.s, ordering),
        }
    }

    pub fn refresh(&self, ordering: Ordering) {
        self.store(CoarseInstant::now(), ordering)
    }

    pub fn elapsed(&self, ordering: Ordering) -> CoarseDuration {
        self.load(ordering).elapsed()
    }
}

impl CoarseInstant {
    pub fn now() -> Self {
        let now = Instant::now();
        Self {
            s: (now.ns / NS_PER_SECOND) as u32,
        }
    }

    pub fn recent() -> Self {
        _clock().recent_coarse()
    }

    pub fn elapsed(&self) -> CoarseDuration {
        CoarseInstant::now() - self
    }
}

impl std::ops::Sub<&CoarseInstant> for CoarseInstant {
    type Output = CoarseDuration;

    fn sub(self, other: &CoarseInstant) -> <Self as std::ops::Sub<&CoarseInstant>>::Output {
        CoarseDuration {
            s: self.s - other.s,
        }
    }
}

impl std::ops::Sub<CoarseInstant> for CoarseInstant {
    type Output = CoarseDuration;

    fn sub(self, other: CoarseInstant) -> <Self as std::ops::Sub<CoarseInstant>>::Output {
        self.sub(&other)
    }
}

impl std::ops::Add<&CoarseDuration> for CoarseInstant {
    type Output = CoarseInstant;

    fn add(self, other: &CoarseDuration) -> <Self as std::ops::Add<&CoarseDuration>>::Output {
        CoarseInstant {
            s: self.s + other.s,
        }
    }
}

impl std::ops::Add<CoarseDuration> for CoarseInstant {
    type Output = CoarseInstant;

    fn add(self, other: CoarseDuration) -> <Self as std::ops::Add<CoarseDuration>>::Output {
        self.add(&other)
    }
}

impl std::ops::AddAssign<CoarseDuration> for CoarseInstant {
    fn add_assign(&mut self, other: CoarseDuration) {
        self.s += other.s
    }
}

impl std::ops::Sub<&CoarseDuration> for CoarseInstant {
    type Output = CoarseInstant;

    fn sub(self, other: &CoarseDuration) -> <Self as std::ops::Sub<&CoarseDuration>>::Output {
        CoarseInstant {
            s: self.s - other.s,
        }
    }
}

impl std::ops::Sub<CoarseDuration> for CoarseInstant {
    type Output = CoarseInstant;

    fn sub(self, other: CoarseDuration) -> <Self as std::ops::Sub<CoarseDuration>>::Output {
        self.sub(&other)
    }
}

impl std::ops::SubAssign<CoarseDuration> for CoarseInstant {
    fn sub_assign(&mut self, other: CoarseDuration) {
        self.s -= other.s
    }
}

#[cfg(all(not(target_os = "macos"), not(target_os = "ios"), unix))]
impl Instant {
    pub fn now() -> Self {
        let mut ts = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe {
            libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
        }
        Instant {
            ns: ts.tv_sec as u64 * NS_PER_SECOND + ts.tv_nsec as u64,
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
impl Instant {
    pub fn now() -> Self {
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
                ns: time * info.numer as u64 / info.denom as u64,
            }
        }
    }
}

#[cfg(target_os = "windows")]
impl Instant {
    pub fn now() -> Instant {
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
            ns: (cnt as f64 / (*PRF_FREQUENCY as f64 / 1_000_000_000_f64)) as u64,
        }
    }
}

impl Instant {
    pub fn elapsed(&self) -> Duration {
        Instant::now() - self
    }

    pub fn recent() -> Instant {
        _clock().recent_precise()
    }
}

impl std::ops::Sub<&Instant> for Instant {
    type Output = Duration;

    fn sub(self, other: &Instant) -> <Self as std::ops::Sub<&Instant>>::Output {
        Duration {
            ns: self.ns - other.ns,
        }
    }
}

impl std::ops::Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, other: Instant) -> <Self as std::ops::Sub<Instant>>::Output {
        self.sub(&other)
    }
}

impl std::ops::Add<&Duration> for Instant {
    type Output = Instant;

    fn add(self, other: &Duration) -> <Self as std::ops::Add<&Duration>>::Output {
        Instant {
            ns: self.ns + other.ns,
        }
    }
}

impl std::ops::Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, other: Duration) -> <Self as std::ops::Add<Duration>>::Output {
        self.add(&other)
    }
}

impl std::ops::AddAssign<Duration> for Instant {
    fn add_assign(&mut self, other: Duration) {
        self.ns += other.ns
    }
}

impl std::ops::Sub<&Duration> for Instant {
    type Output = Instant;

    fn sub(self, other: &Duration) -> <Self as std::ops::Sub<&Duration>>::Output {
        Instant {
            ns: self.ns - other.ns,
        }
    }
}

impl std::ops::Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, other: Duration) -> <Self as std::ops::Sub<Duration>>::Output {
        self.sub(&other)
    }
}

impl std::ops::SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, other: Duration) {
        self.ns -= other.ns
    }
}

/// `CoarseDuration` is a lower-resolution version of `Duration`. It represents
/// a period of time with one-second resolution.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct CoarseDuration {
    s: u32,
}

impl CoarseDuration {
    pub const SECOND: CoarseDuration = CoarseDuration::from_secs(1);
    pub const ZERO: CoarseDuration = CoarseDuration::from_secs(0);
    pub const MAX: CoarseDuration = CoarseDuration::from_secs(u32::MAX);

    pub const fn new(secs: u32) -> Self {
        Self { s: secs }
    }

    pub const fn from_secs(secs: u32) -> Self {
        Self::new(secs)
    }

    pub const fn as_sec(&self) -> u32 {
        self.s
    }

    /// Check if the duration spans no time
    pub const fn is_zero(&self) -> bool {
        self.s == 0
    }
}

/// `Duration` is the amount of time between two instants.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Duration {
    ns: u64,
}

impl Duration {
    pub const SECOND: Duration = Duration::from_nanos(NS_PER_SECOND);
    pub const MILLISECOND: Duration = Duration::from_nanos(NS_PER_MILLISECOND);
    pub const MICROSECOND: Duration = Duration::from_nanos(NS_PER_MICROSECOND);
    pub const NANOSECOND: Duration = Duration::from_nanos(1);
    pub const ZERO: Duration = Duration::from_nanos(0);
    pub const MAX: Duration = Duration::from_nanos(u64::MAX);

    /// Create a duration from the specified number of seconds.
    ///
    /// # Panics
    ///
    /// This constructor will panic if the number of seconds exceeds the maximum
    /// representable duration.
    pub fn from_secs(secs: u64) -> Self {
        assert!(secs < u64::MAX / NS_PER_SECOND);
        Self {
            ns: secs * NS_PER_SECOND,
        }
    }

    /// Create a duration from the specified number of milliseconds.
    ///
    /// # Panics
    ///
    /// This constructor will panic if the number of milliseconds exceeds the
    /// maximum representable duration.
    pub fn from_millis(millis: u64) -> Self {
        assert!(millis < u64::MAX / NS_PER_MILLISECOND);
        Self {
            ns: millis * NS_PER_MILLISECOND,
        }
    }

    /// Create a duration from the specified number of microseconds.
    ///
    /// # Panics
    ///
    /// This constructor will panic if the number of microseconds exceeds the
    /// maximum representable duration.
    pub fn from_micros(micros: u64) -> Self {
        assert!(micros < u64::MAX / NS_PER_MICROSECOND);
        Self {
            ns: micros * NS_PER_MICROSECOND,
        }
    }

    /// Create a duration from the specified number of nanoseconds
    pub const fn from_nanos(nanos: u64) -> Self {
        Self { ns: nanos }
    }

    /// Check if the duration spans no time
    pub const fn is_zero(&self) -> bool {
        self.ns == 0
    }

    /// Returns the total duration in fractional seconds
    pub fn as_sec_f64(&self) -> f64 {
        self.ns as f64 / NS_PER_SECOND as f64
    }

    /// Returns the whole number of seconds in the duration
    pub const fn as_sec(&self) -> u64 {
        self.ns / NS_PER_SECOND
    }

    /// Returns the total number of nanoseconds in the duration
    pub const fn as_nanos(&self) -> u64 {
        self.ns
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
        assert!(elapsed.as_sec_f64() >= 1.0);
        assert!(elapsed.as_sec() >= 1);
        assert!(elapsed.as_nanos() >= 1_000_000_000);
    }

    #[test]
    // This tests the 'clock' interface which is hidden behind macros
    fn clock() {
        let now = Instant::now();
        std::thread::sleep(std::time::Duration::new(1, 0));
        let elapsed = now.elapsed();
        assert!(elapsed.as_sec() >= 1);
        assert!(elapsed.as_nanos() >= 1_000_000_000);

        let t0 = Instant::recent();
        let t0_c = Instant::recent();
        std::thread::sleep(std::time::Duration::new(1, 0));
        assert_eq!(Instant::recent(), t0);
        refresh_clock();
        let t1 = Instant::recent();
        let t1_c = Instant::recent();
        assert!((t1 - t0).as_sec_f64() >= 1.0);
        assert!((t1_c - t0_c).as_sec() >= 1);
    }
}
