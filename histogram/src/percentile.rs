// Copyright 2022 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

/// A `Percentile` represents a quantile reading taken from a histogram.
#[derive(Clone, Copy)]
pub struct Percentile {
    pub(crate) percentile: f64,
    pub(crate) bucket: Bucket,
}

impl Percentile {
    /// Returns the percentile represented by this reading, from [0.0 - 100.0]
    pub fn percentile(&self) -> f64 {
        self.percentile
    }

    /// Returns the bucket which contains the reading.
    pub fn bucket(&self) -> Bucket {
        self.bucket
    }
}
