// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::summary::SummaryStruct;
use crate::traits::*;

use rustcommon_atomics::{Atomic, AtomicBool, Ordering};

use std::sync::RwLock;
use std::time::Instant;

/// Internal type which stores fields necessary to track a corresponding
/// statistic.
pub struct Channel<Value, Count>
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
    summary: Option<SummaryStruct<Value, Count>>,
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
    pub fn new(statistic: &dyn Statistic<Value, Count>) -> Self {
        let summary = statistic.summary().map(|v| v.build());
        Self {
            empty: AtomicBool::new(true),
            reading: Default::default(),
            refreshed: RwLock::new(Instant::now()),
            summary,
        }
    }

    /// Records a bucket value + count pair into the summary.
    pub fn record_bucket(
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

    /// Records a counter reading
    pub fn record_counter(&self, time: Instant, value: <Value as Atomic>::Primitive) {
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

    /// Records a gauge reading
    pub fn record_gauge(&self, time: Instant, value: <Value as Atomic>::Primitive) {
        if let Some(summary) = &self.summary {
            summary.increment(time, value, 1_u8.into());
        }
        self.reading.store(value, Ordering::Relaxed);
        if self.empty.load(Ordering::Relaxed) {
            self.empty.store(false, Ordering::Relaxed);
        }
    }

    /// Returns a percentile across stored readings/rates/...
    pub fn percentile(&self, percentile: f64) -> Result<<Value as Atomic>::Primitive, ()> {
        if let Some(summary) = &self.summary {
            summary.percentile(percentile).map_err(|_| ())
        } else {
            Err(())
        }
    }

    /// Returns the main reading for the channel (eg: counter, gauge)
    pub fn reading(&self) -> Result<<Value as Atomic>::Primitive, ()> {
        if !self.empty.load(Ordering::Relaxed) {
            Ok(self.reading.load(Ordering::Relaxed))
        } else {
            Err(())
        }
    }
}
