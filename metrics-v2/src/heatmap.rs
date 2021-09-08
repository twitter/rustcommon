// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Metric;
use rustcommon_atomics::AtomicU64;
use rustcommon_heatmap::AtomicHeatmap;

pub type Heatmap = AtomicHeatmap<u64, AtomicU64>;

impl<V, C> Metric for AtomicHeatmap<V, C>
where
    V: Send + Sync + 'static,
    C: Send + Sync + 'static,
{
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}
