// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

/// Operations that fetch, compare, and store
pub trait FetchCompareStore: Atomic {
    /// Stores the value if it is greater than the current value.
    ///
    /// The return value is the previous value.
    ///
    /// This operation takes an `Ordering` argument which describes the memory
    /// ordering of the operation. All ordering modes are possible. When using
    /// `AcqRel`, the operation might fail and just perform an `Acquire` load,
    /// but not have `Release` semantics. Using `Acquire` makes the store part
    /// `Relaxed` if it happens, and using `Release` makes the load part
    /// `Relaxed`.
    fn fetch_max(
        &self,
        value: <Self as Atomic>::Primitive,
        ordering: Ordering,
    ) -> <Self as Atomic>::Primitive;

    /// Stores the value if it is less than the current value.
    ///
    /// The return value is the previous value.
    ///
    /// This operation takes an `Ordering` argument which describes the memory
    /// ordering of the operation. All ordering modes are possible. When using
    /// `AcqRel`, the operation might fail and just perform an `Acquire` load,
    /// but not have `Release` semantics. Using `Acquire` makes the store part
    /// `Relaxed` if it happens, and using `Release` makes the load part
    /// `Relaxed`.
    fn fetch_min(
        &self,
        value: <Self as Atomic>::Primitive,
        ordering: Ordering,
    ) -> <Self as Atomic>::Primitive;
}
