// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_histogram::HistogramError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum HeatmapError {
    #[error("heatmap contains no samples")]
    Empty,
    #[error("invalid percentile")]
    InvalidPercentile,
    #[error("value out of range")]
    OutOfRange,
}

impl From<HistogramError> for HeatmapError {
	fn from(other: HistogramError) -> Self {
		match other {
			HistogramError::Empty => Self::Empty,
			HistogramError::InvalidPercentile => Self::InvalidPercentile,
			HistogramError::OutOfRange => Self::OutOfRange,
		}
	}
}
