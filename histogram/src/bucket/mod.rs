// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Counter;

/// A bucket stores a count across a range of values
pub struct Bucket<Value, Count> {
    pub(crate) min: Value,
    pub(crate) max: Value,
    pub(crate) value: Value,
    pub(crate) count: Count,
}

impl<Value, Count> Bucket<Value, Count>
where
    Value: Copy + std::ops::Sub<Output = Value>,
    Count: Counter,
{
    /// Return the minimum value storable in the `Bucket`
    pub fn min(&self) -> Value {
        self.min
    }

    /// Return the nominal value for the `Bucket`
    pub fn value(&self) -> Value {
        self.value
    }

    /// Return the count of values recorded into this `Bucket`
    pub fn count(&self) -> Count {
        self.count
    }

    /// Returns the range of values storable in this `Bucket`
    pub fn width(&self) -> Value {
        self.max - self.min
    }
}
