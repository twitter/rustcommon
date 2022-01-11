// Copyright 2022 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICEN

use crate::*;

/// An estimate of Unix Time based on applying a fixed offset to the `Instant`
/// type. Unlike `std::time::SystemTime` this type is guaranteed to be
/// monotonically non-decreasing.
///
/// It is important to note that while `UnixInstant`s will have ordering
/// matching real time ordering, they are not guaranteed to be phase accurate
/// or steady. This means that if the system clock is set wrong and/or ticking
/// at an unusual rate, that comparisons to external `UnixInstant`s are not
/// guaranteed to be correct through the life of the program.
#[repr(transparent)]
pub struct UnixInstant<T> {
    pub(crate) inner: T,
}

impl<T> Eq for UnixInstant<T> where T: Eq {}

impl<T> PartialEq for UnixInstant<T>
where
    T: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.inner.eq(&rhs.inner)
    }
}

impl<T> Ord for UnixInstant<T>
where
    T: Ord,
{
    fn cmp(&self, rhs: &Self) -> core::cmp::Ordering {
        self.inner.cmp(&rhs.inner)
    }
}

impl<T> core::hash::Hash for UnixInstant<T>
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

impl<T> PartialOrd for UnixInstant<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, rhs: &Self) -> Option<core::cmp::Ordering> {
        self.inner.partial_cmp(&rhs.inner)
    }
}

impl<T> Clone for UnixInstant<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Copy for UnixInstant<T> where T: Copy {}

impl UnixInstant<Seconds<u32>> {
    pub fn now() -> Self {
        let mut ts = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe {
            libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
        }

        let coarse = ts.tv_sec as u32 + CLOCK.coarse_offset.load(Ordering::Relaxed).inner.inner;

        UnixInstant {
            inner: Seconds { inner: coarse },
        }
    }

    pub fn recent() -> Self {
        CLOCK.initialize();
        let recent = CLOCK.coarse.load(Ordering::Relaxed).inner.inner
            + CLOCK.coarse_offset.load(Ordering::Relaxed).inner.inner;
        UnixInstant {
            inner: Seconds { inner: recent },
        }
    }
}

impl core::fmt::Debug for UnixInstant<Seconds<u32>> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UnixInstant<Seconds<u32>>")
            .field("secs", &self.inner.inner)
            .finish()
    }
}

instant!(UnixInstant<Seconds<u32>>);
atomic!(UnixInstant<Seconds<AtomicU32>>, Seconds<u32>);

impl UnixInstant<Nanoseconds<u64>> {
    pub fn now() -> Self {
        let mut ts = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe {
            libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
        }

        let precise = ts.tv_sec as u64 * NANOS_PER_SEC
            + CLOCK.precise_offset.load(Ordering::Relaxed).inner.inner;

        UnixInstant {
            inner: Nanoseconds { inner: precise },
        }
    }

    pub fn recent() -> Self {
        CLOCK.initialize();
        let recent = CLOCK.precise.load(Ordering::Relaxed).inner.inner
            + CLOCK.precise_offset.load(Ordering::Relaxed).inner.inner;
        UnixInstant {
            inner: Nanoseconds { inner: recent },
        }
    }
}

impl core::fmt::Debug for UnixInstant<Nanoseconds<u64>> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UnixInstant<Nanoseconds<u64>>")
            .field("nanos", &self.inner.inner)
            .finish()
    }
}

instant!(UnixInstant<Nanoseconds<u64>>);
atomic!(UnixInstant<Nanoseconds<AtomicU64>>, Nanoseconds<u64>);
