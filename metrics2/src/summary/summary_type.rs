// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::traits::*;
use rustcommon_atomics::Atomic;
use rustcommon_heatmap::AtomicHeatmap;
use rustcommon_streamstats::AtomicStreamstats;
use std::time::Instant;

pub(crate) enum SummaryType<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    Heatmap(AtomicHeatmap<<Value as Atomic>::Primitive, Count>),
    Stream(AtomicStreamstats<Value>),
}

impl<Value, Count> SummaryType<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    pub fn increment(
        &self,
        time: Instant,
        value: <Value as Atomic>::Primitive,
        count: <Count as Atomic>::Primitive,
    ) {
        match self {
            Self::Heatmap(heatmap) => heatmap.increment(time, value, count),
            Self::Stream(stream) => stream.insert(value),
        }
    }

    pub fn percentile(&self, percentile: f64) -> Result<<Value as Atomic>::Primitive, ()> {
        match self {
            Self::Heatmap(heatmap) => heatmap.percentile(percentile).map_err(|_| ()),
            Self::Stream(stream) => stream.percentile(percentile).map_err(|_| ()),
        }
    }
}
