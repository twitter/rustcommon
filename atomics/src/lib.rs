// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! A collection of atomic types which are unified through traits to allow for
//! use as generic types in other datastructures. Also provides non standard
//! atomic types such as an atomic `Option` type.

#![deny(clippy::all)]

#[macro_use]
mod macros;

mod traits;
mod types;

// mod atomic_counter;
// mod atomic_primitive;

pub use crate::traits::*;
pub use crate::types::*;

// pub use crate::atomic_counter::*;
// pub use crate::atomic_primitive::*;
pub use core::sync::atomic::Ordering;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usize() {
        let x = AtomicUsize::new(0);
        assert_eq!(x.load(Ordering::SeqCst), 0_usize);
        x.store(42, Ordering::SeqCst);
        assert_eq!(x.load(Ordering::SeqCst), 42_usize);
    }
}
