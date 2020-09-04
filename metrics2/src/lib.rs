// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::ops::Sub;
use dashmap::DashMap;
use rustcommon_streamstats::AtomicStreamstats;
use std::sync::RwLock;
use std::time::Duration;
use std::time::Instant;

use rustcommon_heatmap::*;

pub trait Value: Atomic + Default {}
pub trait Count: Atomic + Default + AtomicCounter {}
pub trait Primitive:
    Ord + Indexing + Copy + From<u8> + Sub<Self, Output = Self> + FloatConvert
{
}

impl Primitive for u8 {}
impl Primitive for u16 {}
impl Primitive for u32 {}
impl Primitive for u64 {}

impl Count for AtomicU8 {}
impl Count for AtomicU16 {}
impl Count for AtomicU32 {}
impl Count for AtomicU64 {}

impl Value for AtomicU8 {}
impl Value for AtomicU16 {}
impl Value for AtomicU32 {}
impl Value for AtomicU64 {}

/// A collection of channels which each record measurements for a corresponding
/// `Statistic`. It is designed for concurrent access, making it useful for
/// providing a unified metrics library for multi-threaded applications.
pub struct Metrics<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    channels: DashMap<String, Channel<Value, Count>>,
}

impl<Value, Count> Metrics<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    /// Create a new empty metrics registry
    pub fn new() -> Self {
        Self {
            channels: DashMap::new(),
        }
    }

    /// Register a new statistic.
    pub fn register(&self, statistic: &dyn Statistic<Value, Count>) {
        if !self.channels.contains_key(statistic.name()) {
            let channel = Channel::new(statistic);
            self.channels.insert(statistic.name().to_string(), channel);
        }
    }

    /// Record a bucket value + count pair for distribution based statistics.
    pub fn record_bucket(
        &self,
        statistic: &dyn Statistic<Value, Count>,
        time: Instant,
        value: <Value as Atomic>::Primitive,
        count: <Count as Atomic>::Primitive,
    ) -> Result<(), ()> {
        if statistic.source() == Source::Distribution {
            if let Some(channel) = self.channels.get(statistic.name()) {
                channel.record_bucket(time, value, count)
            } else {
                // statistic not registered
                Err(())
            }
        } else {
            // source mismatch
            Err(())
        }
    }

    /// Record a counter observation for counter based statistics.
    pub fn record_counter(
        &self,
        statistic: &dyn Statistic<Value, Count>,
        time: Instant,
        value: <Value as Atomic>::Primitive,
    ) -> Result<(), ()> {
        if statistic.source() == Source::Counter {
            if let Some(channel) = self.channels.get(statistic.name()) {
                channel.record_counter(time, value);
                Ok(())
            } else {
                // statistic not registered
                Err(())
            }
        } else {
            // source mismatch
            Err(())
        }
    }

    /// Record a gauge observation for gauge based statistics.
    pub fn record_gauge(
        &self,
        statistic: &dyn Statistic<Value, Count>,
        time: Instant,
        value: <Value as Atomic>::Primitive,
    ) -> Result<(), ()> {
        if statistic.source() == Source::Gauge {
            if let Some(channel) = self.channels.get(statistic.name()) {
                channel.record_gauge(time, value);
                Ok(())
            } else {
                // statistic not registered
                Err(())
            }
        } else {
            // source mismatch
            Err(())
        }
    }

    /// Return a percentile for the given statistic. For counters, it is the
    /// percentile of secondly rates across the summary. For gauges, it is the
    /// percentile of gauge readings observed across the summary. For
    /// distributions it is the percentile across the configured summary.
    pub fn percentile(
        &self,
        statistic: &dyn Statistic<Value, Count>,
        percentile: f64,
    ) -> Result<<Value as Atomic>::Primitive, ()> {
        if let Some(channel) = self.channels.get(statistic.name()) {
            channel.percentile(percentile).map_err(|_| ())
        } else {
            Err(())
        }
    }

    /// Return the reading for the statistic. For counters and gauges, this is
    /// the most recent measurement recorded.
    // TODO: decide on how to handle distribution channels
    pub fn reading(
        &self,
        statistic: &dyn Statistic<Value, Count>,
    ) -> Result<<Value as Atomic>::Primitive, ()> {
        if let Some(channel) = self.channels.get(statistic.name()) {
            channel.reading()
        } else {
            Err(())
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum Source {
    /// Indicates that the source is a monotonically incrementing count.
    Counter,
    /// Indicates that the source is an instantaneous gauge reading.
    Gauge,
    /// Indicates that the source is an underlying distribution (histogram).
    Distribution,
}

pub trait Statistic<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    /// The name is used to lookup the channel for the statistic and should be
    /// unique for each statistic.
    fn name(&self) -> &str;
    /// Indicates which source type the statistic tracks.
    fn source(&self) -> Source;
    /// Optionally, specify a summary type which is used to produce percentiles.
    fn summary(&self) -> Option<Summary<Value, Count>> {
        None
    }
}

/// Internal type which stores fields necessary to track a corresponding
/// statistic.
struct Channel<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    refreshed: RwLock<Instant>,
    empty: AtomicBool,
    reading: Value,
    summary: Option<SummaryType<Value, Count>>,
}

impl<Value, Count> Channel<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    /// Creates an empty channel for a statistic.
    fn new(statistic: &dyn Statistic<Value, Count>) -> Self {
        let summary = statistic.summary().map(|v| v.inner);
        Self {
            empty: AtomicBool::new(true),
            reading: Default::default(),
            refreshed: RwLock::new(Instant::now()),
            summary,
        }
    }

    /// Records a bucket value + count pair into the summary.
    fn record_bucket(
        &self,
        time: Instant,
        value: <Value as Atomic>::Primitive,
        count: <Count as Atomic>::Primitive,
    ) -> Result<(), ()> {
        if let Some(summary) = &self.summary {
            summary.increment(time, value, count);
            Ok(())
        } else {
            Err(())
        }
    }

    fn record_counter(&self, time: Instant, value: <Value as Atomic>::Primitive) {
        if !self.empty.load(Ordering::Relaxed) {
            if let Some(summary) = &self.summary {
                let t0 = self.refreshed.write().unwrap();
                let v0 = self.reading.load(Ordering::Relaxed);
                let dt = time - *t0;
                let dv = (value - v0).to_float();
                let rate = (dv
                    / (dt.as_secs() as f64 + dt.subsec_nanos() as f64 / 1_000_000_000.0))
                    .ceil();
                summary.increment(
                    time,
                    <Value as Atomic>::Primitive::from_float(rate),
                    1_u8.into(),
                );
            }
            self.reading.store(value, Ordering::Relaxed);
        } else {
            self.reading.store(value, Ordering::Relaxed);
            self.empty.store(false, Ordering::Relaxed);
        }
    }

    fn record_gauge(&self, time: Instant, value: <Value as Atomic>::Primitive) {
        if let Some(summary) = &self.summary {
            summary.increment(time, value, 1_u8.into());
        }
        self.reading.store(value, Ordering::Relaxed);
        if self.empty.load(Ordering::Relaxed) {
            self.empty.store(false, Ordering::Relaxed);
        }
    }

    fn percentile(&self, percentile: f64) -> Result<<Value as Atomic>::Primitive, ()> {
        if let Some(summary) = &self.summary {
            summary.percentile(percentile).map_err(|_| ())
        } else {
            Err(())
        }
    }

    fn reading(&self) -> Result<<Value as Atomic>::Primitive, ()> {
        if !self.empty.load(Ordering::Relaxed) {
            Ok(self.reading.load(Ordering::Relaxed))
        } else {
            Err(())
        }
    }
}

pub trait FloatConvert {
    fn to_float(self) -> f64;
    fn from_float(value: f64) -> Self;
}

impl FloatConvert for u64 {
    fn to_float(self) -> f64 {
        self as f64
    }
    fn from_float(value: f64) -> Self {
        value as Self
    }
}

impl FloatConvert for u32 {
    fn to_float(self) -> f64 {
        self as f64
    }
    fn from_float(value: f64) -> Self {
        value as Self
    }
}

impl FloatConvert for u16 {
    fn to_float(self) -> f64 {
        self as f64
    }
    fn from_float(value: f64) -> Self {
        value as Self
    }
}

impl FloatConvert for u8 {
    fn to_float(self) -> f64 {
        self as f64
    }
    fn from_float(value: f64) -> Self {
        value as Self
    }
}

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
    inner: SummaryType<Value, Count>,
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

enum SummaryType<Value, Count>
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
    fn increment(
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

    fn percentile(&self, percentile: f64) -> Result<<Value as Atomic>::Primitive, ()> {
        match self {
            Self::Heatmap(heatmap) => heatmap.percentile(percentile).map_err(|_| ()),
            Self::Stream(stream) => stream.percentile(percentile).map_err(|_| ()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    enum TestStat {
        Alpha,
    }

    impl Statistic<AtomicU64, AtomicU64> for TestStat {
        fn name(&self) -> &str {
            match self {
                Self::Alpha => "alpha",
            }
        }

        fn source(&self) -> Source {
            match self {
                Self::Alpha => Source::Counter,
            }
        }

        fn summary(&self) -> Option<Summary<AtomicU64, AtomicU64>> {
            match self {
                Self::Alpha => Some(Summary::stream(1000)),
            }
        }
    }

    #[test]
    fn basic() {
        let metrics = Metrics::<AtomicU64, AtomicU64>::new();
        metrics.register(&TestStat::Alpha);
        assert!(metrics.reading(&TestStat::Alpha).is_err());
        metrics
            .record_counter(&TestStat::Alpha, Instant::now(), 0)
            .expect("failed to record counter");
        assert_eq!(metrics.reading(&TestStat::Alpha), Ok(0));
        let now = Instant::now();
        metrics
            .record_counter(&TestStat::Alpha, now, 0)
            .expect("failed to record counter");
        assert_eq!(metrics.reading(&TestStat::Alpha), Ok(0));
        assert_eq!(metrics.percentile(&TestStat::Alpha, 0.0), Ok(0));
        metrics
            .record_counter(&TestStat::Alpha, now + Duration::from_millis(1000), 1)
            .expect("failed to record counter");
        assert_eq!(metrics.reading(&TestStat::Alpha), Ok(1));
        assert_eq!(metrics.percentile(&TestStat::Alpha, 100.0), Ok(1));
    }
}
