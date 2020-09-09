// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_atomics::*;
use rustcommon_heatmap::AtomicCounter;

pub trait Count: Atomic + Default + AtomicCounter {}

impl Count for AtomicU8 {}
impl Count for AtomicU16 {}
impl Count for AtomicU32 {}
impl Count for AtomicU64 {}
