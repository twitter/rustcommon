// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

/// Arithmetic operations on atomic types
pub trait Arithmetic: Atomic {
    /// Adds to the current value, returning the previous value.
    ///
    /// This operation wraps around on overflow.
    ///
    /// This operation takes an `Ordering` argument which describes the memory
    /// ordering of the operation. All ordering modes are possible. Using
    /// `Acquire` makes the store part of the operation `Relaxed`, and using
    /// `Release` makes the load part of the operation `Relaxed`.
    fn fetch_add(
        &self,
        value: <Self as Atomic>::Primitive,
        ordering: Ordering,
    ) -> <Self as Atomic>::Primitive;

    /// Subtracts from the current value, returning the previous value.
    ///
    /// This operation wraps around on overflow.
    ///
    /// This operation takes an `Ordering` argument which describes the memory
    /// ordering of the operation. All ordering modes are possible. Using
    /// `Acquire` makes the store part of the operation `Relaxed`, and using
    /// `Release` makes the load part of the operation `Relaxed`.
    fn fetch_sub(
        &self,
        value: <Self as Atomic>::Primitive,
        ordering: Ordering,
    ) -> <Self as Atomic>::Primitive;
}
