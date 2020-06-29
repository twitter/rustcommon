// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

/// Common operations on atomic types
pub trait Atomic {
    type Primitive;

    /// Creates a new atomic type from a primitive type.
    // fn new(value: T) -> Self;

    /// Loads the value from the atomic type.
    ///
    /// `load` takes an `Ordering` argument which describes the memory ordering
    /// of this operation. Possible values are `SeqCst`, `Acquire`, and
    /// `Relaxed`
    fn load(&self, order: Ordering) -> Self::Primitive;

    /// Stores a value into the atomic type.
    ///
    /// `store` takes an `Ordering` argument which describes the memory ordering
    /// of this operation. Possible values are `SeqCst`, `Acquire`, `Release`,
    /// and `Relaxed`.
    fn store(&self, value: Self::Primitive, order: Ordering);

    /// Stores a value into the atomic type, returning the previous value.
    /// `swap` takes an `Ordering` argument which describes the memory ordering
    /// of this operation. All ordering modes are possible. Note that using
    /// `Acquire` makes the store part of this operation `Relaxed`, and using
    /// `Release` makes the load part `Relaxed`.
    fn swap(&self, value: Self::Primitive, order: Ordering) -> Self::Primitive;

    /// Stores a value into the atomic type if the current value is the same as
    /// the `current` value.
    ///
    /// The return value is always the previous value. If it is equal to
    /// `current`, then the value was updated.
    ///
    /// `compare_and_swap` takes an `Ordering` argument which describes the
    /// memory ordering of this operation. Note that even when using `AcqRel`,
    /// the operation might fail and hence just perform an `Acquire` load, but
    /// not have `Release` semantics. Using `Acquire` makes the store part of
    /// this operation `Relaxed` if it happens, and using `Release` makes the
    /// load part `Relaxed`.
    fn compare_and_swap(
        &self,
        current: Self::Primitive,
        new: Self::Primitive,
        order: Ordering,
    ) -> Self::Primitive;

    /// Stores a value into the atomic type if the current value is the same as
    /// as the `current` value.
    ///
    /// The return value is a result indicating whether the new value was
    /// written and containing the previous value. On success this value is
    /// guaranteed to be equal to `current`.
    ///
    /// `compare_exchange` takes two `Ordering` arguments to describe the
    /// memory ordering of this operation. The first describes the required
    /// ordering if the operation succeeds while the second describes the
    /// required ordering when the operation fails. Using `Acquire` as success
    /// ordering makes the store part of this operation `Relaxed`, and using
    /// `Release` makes the successful load `Relaxed`. The failure ordering
    /// can only be `SeqCst`, `Acquire`, or `Relaxed` and must be equivalent
    /// to or weaker than the success ordering.
    fn compare_exchange(
        &self,
        current: Self::Primitive,
        new: Self::Primitive,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self::Primitive, Self::Primitive>;

    /// Stores a value into the atomic type if the current value is the same as
    /// as the `current` value.
    ///
    /// Unlike `compare_exchange`, this function is allowed to spuriously fail
    /// even when the comparison succeeds, which can result in more efficient
    /// code on some platforms. The return value is a result indicating whether
    /// the new value was written and containing the previous value.
    ///
    /// `compare_exchange_weak` takes two `Ordering` arguments to describe the
    /// memory ordering of this operation. The first describes the required
    /// ordering if the operation succeeds while the second describes the
    /// required ordering when the operation fails. Using `Acquire` as success
    /// ordering makes the store part of this operation `Relaxed`, and using
    /// `Release` makes the successful load `Relaxed`. The failure ordering
    /// can only be `SeqCst`, `Acquire`, or `Relaxed` and must be equivalent
    /// to or weaker than the success ordering.
    fn compare_exchange_weak(
        &self,
        current: Self::Primitive,
        new: Self::Primitive,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self::Primitive, Self::Primitive>;
}
