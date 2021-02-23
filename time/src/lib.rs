use core::sync::atomic::AtomicU32;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;

const NS_PER_SECOND: u64 = 1_000_000_000;

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

/// `AtomicCoarseInstant` is an opaque type that represents a moment in time to
/// the nearest second. Unlike `CoarseInstant`, it is thread-safe.
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
            s: (now.ns as f64 / NS_PER_SECOND as f64).round() as u32
        }
    }
}


impl CoarseInstant {
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
pub struct CoarseDuration {
    s: u32,
}

impl CoarseDuration {
    pub fn as_sec(&self) -> u32 {
        self.s
    }
}

/// `Duration` is the amount of time between two instants.
pub struct Duration {
    ns: u64,
}

impl Duration {
    pub fn as_sec_f64(&self) -> f64 {
        self.ns as f64 / NS_PER_SECOND as f64
    }

    pub fn as_sec(&self) -> u64 {
        self.ns / NS_PER_SECOND
    }

    pub fn as_nanos(&self) -> u64 {
        self.ns
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works() {
        let now = Instant::now();
        std::thread::sleep(std::time::Duration::new(1, 0));
        let elapsed = now.elapsed();
        assert!(elapsed.as_sec_f64() >= 1.0);
    }
}
