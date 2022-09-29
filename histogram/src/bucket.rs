// Copyright 2022 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

/// A `Bucket` represents a discrete range of values and the sum of recorded
/// counts within this range.
#[derive(Clone, Copy)]
pub struct Bucket {
    pub(crate) low: u64,
    pub(crate) high: u64,
    pub(crate) count: u32,
}

impl Bucket {
    /// The lowest value represented by this `Bucket`.
    pub fn low(&self) -> u64 {
        self.low
    }

    /// The highest value represented by this `Bucket`.
    pub fn high(&self) -> u64 {
        self.high
    }

    /// The sum of the recorded counts which fall into this `Bucket`.
    pub fn count(&self) -> u32 {
        self.count
    }
}
