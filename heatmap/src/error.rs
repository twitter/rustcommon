// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use histogram::Error as HistogramError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("heatmap contains no samples")]
    Empty,
    #[error("invalid percentile")]
    InvalidPercentile,
    #[error("value out of range")]
    OutOfRange,
}

impl From<HistogramError> for Error {
    fn from(other: HistogramError) -> Self {
        match other {
            HistogramError::Empty => Self::Empty,
            HistogramError::InvalidPercentile => Self::InvalidPercentile,
            HistogramError::OutOfRange => Self::OutOfRange,
        }
    }
}
