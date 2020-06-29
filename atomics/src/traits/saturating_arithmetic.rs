// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

/// Saturating arithmetic on atomic types
pub trait SaturatingArithmetic: Atomic {
    /// Adds to the current value, returning the previous value.
    ///
    /// This operation saturates at the numeric bound.
    ///
    /// This operation takes an `Ordering` argument which describes the memory
    /// ordering of the operation. All ordering modes are possible. When using
    /// `AcqRel`, the operation might fail and just perform an `Acquire` load,
    /// but not have `Release` semantics. Using `Acquire` makes the store part
    /// `Relaxed` if it happens, and using `Release` makes the load part
    /// `Relaxed`.
    fn fetch_saturating_add(
        &self,
        value: <Self as Atomic>::Primitive,
        ordering: Ordering,
    ) -> <Self as Atomic>::Primitive;

    /// Subtracts from the current value, returning the previous value.
    ///
    /// This operation saturates at the numeric bound.
    ///
    /// This operation takes an `Ordering` argument which describes the memory
    /// ordering of the operation. All ordering modes are possible. When using
    /// `AcqRel`, the operation might fail and just perform an `Acquire` load,
    /// but not have `Release` semantics. Using `Acquire` makes the store part
    /// `Relaxed` if it happens, and using `Release` makes the load part
    /// `Relaxed`.
    fn fetch_saturating_sub(
        &self,
        value: <Self as Atomic>::Primitive,
        ordering: Ordering,
    ) -> <Self as Atomic>::Primitive;
}
