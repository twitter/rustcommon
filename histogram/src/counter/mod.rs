// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub trait AtomicCounter:
    rustcommon_atomics::Atomic
    + rustcommon_atomics::Unsigned
    + rustcommon_atomics::SaturatingArithmetic
    + Default
{
}

impl AtomicCounter for rustcommon_atomics::AtomicU8 {}
impl AtomicCounter for rustcommon_atomics::AtomicU16 {}
impl AtomicCounter for rustcommon_atomics::AtomicU32 {}
impl AtomicCounter for rustcommon_atomics::AtomicU64 {}
impl AtomicCounter for rustcommon_atomics::AtomicUsize {}

pub trait Counter: Default + Copy {
    fn saturating_add(&mut self, value: Self);
    fn saturating_sub(&mut self, value: Self);
}
impl Counter for u8 {
    fn saturating_add(&mut self, value: Self) {
        *self = (*self as u8).saturating_add(value);
    }

    fn saturating_sub(&mut self, value: Self) {
        *self = (*self as u8).saturating_sub(value);
    }
}
impl Counter for u16 {
    fn saturating_add(&mut self, value: Self) {
        *self = (*self as u16).saturating_add(value);
    }

    fn saturating_sub(&mut self, value: Self) {
        *self = (*self as u16).saturating_sub(value);
    }
}
impl Counter for u32 {
    fn saturating_add(&mut self, value: Self) {
        *self = (*self as u32).saturating_add(value);
    }

    fn saturating_sub(&mut self, value: Self) {
        *self = (*self as u32).saturating_sub(value);
    }
}
impl Counter for u64 {
    fn saturating_add(&mut self, value: Self) {
        *self = (*self as u64).saturating_add(value);
    }

    fn saturating_sub(&mut self, value: Self) {
        *self = (*self as u64).saturating_sub(value);
    }
}
impl Counter for usize {
    fn saturating_add(&mut self, value: Self) {
        *self = (*self as usize).saturating_add(value);
    }

    fn saturating_sub(&mut self, value: Self) {
        *self = (*self as usize).saturating_sub(value);
    }
}
