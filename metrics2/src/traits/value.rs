// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_atomics::*;

pub trait Value: Atomic + Default {}

impl Value for AtomicU8 {}
impl Value for AtomicU16 {}
impl Value for AtomicU32 {}
impl Value for AtomicU64 {}
