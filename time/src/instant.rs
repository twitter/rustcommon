// Copyright 2022 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

/// The measurement of a monotonically nondecreasing clock. The internal
/// representation is the duration since an arbitrary epoch. Opaque and only
/// useful with other `Instant`s and the `Duration` types.
///
/// It is important to note that the underlying clock is not guaranteed to be
/// steady. It is subject only to frequency corrections.
#[repr(transparent)]
pub struct Instant<T> {
    pub(crate) inner: T,
}

impl<T> Eq for Instant<T> where T: Eq {}

impl<T> PartialEq for Instant<T>
where
    T: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.inner.eq(&rhs.inner)
    }
}

impl<T> Ord for Instant<T>
where
    T: Ord,
{
    fn cmp(&self, rhs: &Self) -> core::cmp::Ordering {
        self.inner.cmp(&rhs.inner)
    }
}

impl<T> PartialOrd for Instant<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, rhs: &Self) -> Option<core::cmp::Ordering> {
        self.inner.partial_cmp(&rhs.inner)
    }
}

impl<T> core::hash::Hash for Instant<T>
where
    T: core::hash::Hash,
{
    fn hash<H>(&self, h: &mut H)
    where
        H: core::hash::Hasher,
    {
        self.inner.hash(h)
    }
}

impl<T> Clone for Instant<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Copy for Instant<T> where T: Copy {}

impl Instant<Seconds<u32>> {
    pub fn now() -> Self {
        let mut ts = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe {
            libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
        }

        Self {
            inner: Seconds::from(ts),
        }
    }

    pub fn recent() -> Self {
        CLOCK.initialize();
        CLOCK.coarse.load(Ordering::Relaxed)
    }
}

impl core::fmt::Debug for Instant<Seconds<u32>> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Instant<Seconds<u32>>")
            .field("secs", &self.inner.inner)
            .finish()
    }
}

instant!(Instant<Seconds<u32>>);

impl Instant<Seconds<AtomicU32>> {
    pub fn now() -> Self {
        Self::new(Instant::<Seconds<u32>>::now())
    }

    pub fn recent() -> Self {
        Self::new(Instant::<Seconds<u32>>::recent())
    }
}

atomic!(Instant<Seconds<AtomicU32>>, Seconds<u32>);

impl Instant<Nanoseconds<u64>> {
    pub fn now() -> Self {
        let mut ts = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe {
            libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
        }

        Self {
            inner: Nanoseconds::from(ts)
        }
    }

    pub fn recent() -> Self {
        CLOCK.initialize();
        CLOCK.precise.load(Ordering::Relaxed)
    }
}

impl core::fmt::Debug for Instant<Nanoseconds<u64>> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Instant<Nanoseconds<u64>>")
            .field("nanos", &self.inner.inner)
            .finish()
    }
}

instant!(Instant<Nanoseconds<u64>>);

impl Instant<Nanoseconds<AtomicU64>> {
    pub fn now() -> Self {
        Self::new(Instant::<Nanoseconds<u64>>::now())
    }

    pub fn recent() -> Self {
        Self::new(Instant::<Nanoseconds<u64>>::recent())
    }
}

atomic!(Instant<Nanoseconds<AtomicU64>>, Nanoseconds<u64>);
atomic_arithmetic!(Instant<Nanoseconds<AtomicU64>>, Duration<Nanoseconds<u64>>);
