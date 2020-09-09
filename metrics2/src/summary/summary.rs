// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::summary::SummaryType;
use crate::traits::*;

use rustcommon_atomics::Atomic;
use rustcommon_heatmap::AtomicHeatmap;
use rustcommon_streamstats::AtomicStreamstats;

use core::time::Duration;

/// Specifies what type of structure should be used for producing summary
/// metrics. Heatmaps are ideal for data that is sourced from histograms.
/// Streams are intended for use with data sourced from a counter / gauge and
/// generally use less memory than a heatmap would unless the range is small and
/// update frequency is very high. For example, for 60 one second windows, a
/// heatmap will use approximately 305kB to store values from 0-1billion with
/// 2 significant figures preserved. This is regardless of the total number of
/// value + count pairs recorded. Conversely, a stream summary requires about
/// 16B per stored value. This would mean that until about 325 samples/s with a
/// 60s window, the stream will use less memory. For 1 sample/s, a stream will
/// use only ~1kB instead of the ~305kB required to maintain a heatmap. If 3
/// significant figures are required, a counter would need to be stored about
/// 3250 times per second for a heatmap to make sense.  
pub struct Summary<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    pub(crate) inner: SummaryType<Value, Count>,
}

impl<Value, Count> Summary<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    /// Specify a heatmap summary will be used. Ideal for data sourced from
    /// histograms.
    pub fn heatmap(
        max: <Value as Atomic>::Primitive,
        precision: u8,
        windows: usize,
        resolution: Duration,
    ) -> Summary<Value, Count> {
        Summary {
            inner: SummaryType::Heatmap(AtomicHeatmap::<<Value as Atomic>::Primitive, Count>::new(
                max, precision, windows, resolution,
            )),
        }
    }

    /// Specify that a stream summary will be used. Stores exactly capacity
    /// previous values.
    pub fn stream(capacity: usize) -> Summary<Value, Count> {
        Summary {
            inner: SummaryType::Stream(AtomicStreamstats::<Value>::new(capacity)),
        }
    }
}
