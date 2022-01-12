// Copyright 2022 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICEN

use crate::*;
use core::ops::AddAssign;

#[repr(transparent)]
pub struct Duration<T> {
    pub(crate) inner: T,
}

impl<T> Eq for Duration<T> where T: Eq {}

impl<T> PartialEq for Duration<T>
where
    T: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.inner.eq(&rhs.inner)
    }
}

impl<T> Ord for Duration<T>
where
    T: Ord,
{
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&rhs.inner)
    }
}

impl<T> PartialOrd for Duration<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        self.inner.partial_cmp(&rhs.inner)
    }
}

impl<T> AddAssign for Duration<T>
where
    T: AddAssign,
{
    fn add_assign(&mut self, other: Self) {
        self.inner += other.inner;
    }
}

impl<T> Clone for Duration<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Copy for Duration<T> where T: Copy {}

impl Duration<Seconds<u32>> {
    pub const SECOND: Self = Self {
        inner: Seconds { inner: 1 },
    };
    pub const ZERO: Self = Self {
        inner: Seconds { inner: 0 },
    };
    pub const MAX: Self = Self {
        inner: Seconds { inner: u32::MAX },
    };

    pub const fn from_secs(seconds: u32) -> Self {
        Self {
            inner: Seconds { inner: seconds },
        }
    }

    pub fn as_secs_f64(&self) -> f64 {
        self.inner.inner as f64
    }

    pub fn as_nanos(&self) -> u64 {
        self.inner.inner as u64 * NANOS_PER_SEC
    }

    pub fn as_secs(&self) -> u32 {
        self.inner.inner
    }
}

impl core::fmt::Debug for Duration<Seconds<u32>> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Duration<Seconds<u32>>")
            .field("secs", &self.inner.inner)
            .finish()
    }
}

impl Duration<Seconds<AtomicU32>> {
    pub const fn from_secs(seconds: u32) -> Self {
        Self {
            inner: Seconds {
                inner: AtomicU32::new(seconds),
            },
        }
    }
}

atomic!(Duration<Seconds<AtomicU32>>, Seconds<u32>);
atomic_arithmetic!(Duration<Seconds<AtomicU32>>, Duration<Seconds<u32>>);

impl Duration<Nanoseconds<u64>> {
    pub const SECOND: Self = Self {
        inner: Nanoseconds {
            inner: NANOS_PER_SEC,
        },
    };
    pub const MILLISECOND: Self = Self {
        inner: Nanoseconds {
            inner: NANOS_PER_MILLI,
        },
    };
    pub const MICROSECOND: Self = Self {
        inner: Nanoseconds {
            inner: NANOS_PER_MICRO,
        },
    };
    pub const NANOSECOND: Self = Self {
        inner: Nanoseconds { inner: 1 },
    };
    pub const ZERO: Self = Self {
        inner: Nanoseconds { inner: 0 },
    };
    pub const MAX: Self = Self {
        inner: Nanoseconds { inner: u64::MAX },
    };

    pub const fn from_nanos(nanoseconds: u64) -> Self {
        Self {
            inner: Nanoseconds { inner: nanoseconds },
        }
    }

    pub fn from_micros(microseconds: u64) -> Self {
        Self {
            inner: Nanoseconds {
                inner: microseconds
                    .checked_mul(NANOS_PER_MICRO)
                    .expect("the specified duration could not be represented with this type"),
            },
        }
    }

    pub fn from_millis(milliseconds: u64) -> Self {
        Self {
            inner: Nanoseconds {
                inner: milliseconds
                    .checked_mul(NANOS_PER_MILLI)
                    .expect("the specified duration could not be represented with this type"),
            },
        }
    }

    pub fn from_secs(seconds: u64) -> Self {
        Self {
            inner: Nanoseconds {
                inner: seconds
                    .checked_mul(NANOS_PER_SEC)
                    .expect("the specified duration could not be represented with this type"),
            },
        }
    }

    pub fn as_secs_f64(&self) -> f64 {
        self.inner.inner as f64 / NANOS_PER_SEC as f64
    }

    pub const fn as_nanos(&self) -> u64 {
        self.inner.inner
    }

    pub const fn as_secs(&self) -> u64 {
        self.inner.inner / NANOS_PER_SEC
    }

    pub const fn subsec_nanos(&self) -> u64 {
        self.inner.inner % NANOS_PER_SEC
    }

    pub const fn as_millis(&self) -> u64 {
        self.inner.inner / NANOS_PER_MILLI
    }

    pub const fn as_micros(&self) -> u64 {
        self.inner.inner / NANOS_PER_MICRO
    }

    pub fn mul_f64(self, rhs: f64) -> Self {
        Self {
            inner: Nanoseconds {
                inner: (self.inner.inner as f64 * rhs) as u64,
            },
        }
    }
}

impl core::fmt::Debug for Duration<Nanoseconds<u64>> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Duration<Nanoseconds<u64>>")
            .field("nanos", &self.inner.inner)
            .finish()
    }
}

impl Duration<Nanoseconds<AtomicU64>> {
    pub const fn from_nanos(nanoseconds: u64) -> Self {
        Self {
            inner: Nanoseconds {
                inner: AtomicU64::new(nanoseconds),
            },
        }
    }
}

atomic!(Duration<Nanoseconds<AtomicU64>>, Nanoseconds<u64>);
atomic_arithmetic!(Duration<Nanoseconds<AtomicU64>>, Duration<Nanoseconds<u64>>);
