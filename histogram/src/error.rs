// Copyright 2022 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use thiserror::Error;

/// Possible errors returned by operations on a histogram.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("histogram contains no samples")]
    /// The histogram contains no samples.
    Empty,
    #[error("invalid percentile")]
    /// The provided percentile is outside of the range 0.0 - 100.0 (inclusive)
    InvalidPercentile,
    #[error("value out of range")]
    /// The provided value is outside of the storable range.
    OutOfRange,
}
