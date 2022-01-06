// Copyright 2022 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICEN

use crate::*;
use core::hash::Hash;

/// A container for time types that stores seconds.
#[repr(transparent)]
pub struct Seconds<T> {
    pub(crate) inner: T,
}

unit!(Seconds<u32>);
atomic!(Seconds<AtomicU32>, u32);
atomic_arithmetic!(Seconds<AtomicU32>, Seconds<u32>);

impl<T> Eq for Seconds<T> where T: Eq {}

impl<T> PartialEq for Seconds<T>
where
    T: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.inner.eq(&rhs.inner)
    }
}

impl<T> Ord for Seconds<T>
where
    T: Ord,
{
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&rhs.inner)
    }
}

impl<T> PartialOrd for Seconds<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        self.inner.partial_cmp(&rhs.inner)
    }
}

impl<T> Hash for Seconds<T>
where
    T: Hash,
{
    fn hash<H>(&self, h: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.inner.hash(h)
    }
}

impl<T> Clone for Seconds<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Copy for Seconds<T> where T: Copy {}

/// A container for time types that stores nanoseconds.
#[repr(transparent)]
pub struct Nanoseconds<T> {
    pub(crate) inner: T,
}

unit!(Nanoseconds<u64>);
atomic!(Nanoseconds<AtomicU64>, u64);
atomic_arithmetic!(Nanoseconds<AtomicU64>, Nanoseconds<u64>);

impl<T> Eq for Nanoseconds<T> where T: Eq {}

impl<T> PartialEq for Nanoseconds<T>
where
    T: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.inner.eq(&rhs.inner)
    }
}

impl<T> Ord for Nanoseconds<T>
where
    T: Ord,
{
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&rhs.inner)
    }
}

impl<T> PartialOrd for Nanoseconds<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        self.inner.partial_cmp(&rhs.inner)
    }
}

impl<T> Hash for Nanoseconds<T>
where
    T: Hash,
{
    fn hash<H>(&self, h: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.inner.hash(h)
    }
}

impl<T> Clone for Nanoseconds<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Copy for Nanoseconds<T> where T: Copy {}

impl From<libc::timespec> for Seconds<u32> {
    fn from(ts: libc::timespec) -> Self {
        Self {
            inner: ts.tv_sec as u32,
        }
    }
}

impl From<libc::timespec> for Seconds<AtomicU32> {
    fn from(ts: libc::timespec) -> Self {
        Self {
            inner: AtomicU32::new(ts.tv_sec as u32),
        }
    }
}

impl From<libc::timespec> for Nanoseconds<u64> {
    fn from(ts: libc::timespec) -> Self {
        Self {
            inner: ts.tv_sec as u64 * NANOS_PER_SEC + ts.tv_nsec as u64,
        }
    }
}

impl From<libc::timespec> for Nanoseconds<AtomicU64> {
    fn from(ts: libc::timespec) -> Self {
        Self {
            inner: AtomicU64::new(ts.tv_sec as u64 * NANOS_PER_SEC + ts.tv_nsec as u64),
        }
    }
}
