// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

/// Bitwise operations on atomic types
pub trait Bitwise: Atomic {
    /// Bitwise "and" with the current value, returning the previous value.
    ///
    /// This operation takes an `Ordering` argument which describes the memory
    /// ordering of the operation. All ordering modes are possible. Using
    /// `Acquire` makes the store part of the operation `Relaxed`, and using
    /// `Release` makes the load part of the operation `Relaxed`.
    fn fetch_and(
        &self,
        value: <Self as Atomic>::Primitive,
        ordering: Ordering,
    ) -> <Self as Atomic>::Primitive;

    /// Bitwise "nand" with the current value, returning the previous value.
    ///
    /// This operation takes an `Ordering` argument which describes the memory
    /// ordering of the operation. All ordering modes are possible. Using
    /// `Acquire` makes the store part of the operation `Relaxed`, and using
    /// `Release` makes the load part of the operation `Relaxed`.
    fn fetch_nand(
        &self,
        value: <Self as Atomic>::Primitive,
        ordering: Ordering,
    ) -> <Self as Atomic>::Primitive;

    /// Bitwise "or" with the current value, returning the previous value.
    ///
    /// This operation takes an `Ordering` argument which describes the memory
    /// ordering of the operation. All ordering modes are possible. Using
    /// `Acquire` makes the store part of the operation `Relaxed`, and using
    /// `Release` makes the load part of the operation `Relaxed`.
    fn fetch_or(
        &self,
        value: <Self as Atomic>::Primitive,
        ordering: Ordering,
    ) -> <Self as Atomic>::Primitive;

    /// Bitwise "xor" with the current value, returning the previous value.
    ///
    /// This operation takes an `Ordering` argument which describes the memory
    /// ordering of the operation. All ordering modes are possible. Using
    /// `Acquire` makes the store part of the operation `Relaxed`, and using
    /// `Release` makes the load part of the operation `Relaxed`.
    fn fetch_xor(
        &self,
        value: <Self as Atomic>::Primitive,
        ordering: Ordering,
    ) -> <Self as Atomic>::Primitive;
}
