// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Metric;

pub use heatmap::Heatmap;

impl Metric for Heatmap {
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}
