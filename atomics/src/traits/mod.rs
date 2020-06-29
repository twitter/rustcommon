// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod arithmetic;
pub use arithmetic::*;

mod atomic;
pub use atomic::*;

mod bitwise;
pub use bitwise::*;

mod fetch_compare_store;
pub use fetch_compare_store::*;

mod saturating_arithmetic;
pub use saturating_arithmetic::*;

// marker traits

/// Values are signed
pub trait Signed {}

/// Values are unsigned
pub trait Unsigned {}
