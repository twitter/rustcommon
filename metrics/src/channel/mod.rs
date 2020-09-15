// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::summary::SummaryStruct;
use crate::traits::*;
use crate::Summary;

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

    /// Updates a counter to a new value if the reading is newer than the stored
    /// reading.
    pub fn record_counter(&self, time: Instant, value: <Value as Atomic>::Primitive) {
        {
            let t0 = self.refreshed.read().unwrap();
            if time <= *t0 {
                return;
            }
        }
        if !self.empty.load(Ordering::Relaxed) {
            if let Some(summary) = &self.summary {
                let mut t0 = self.refreshed.write().unwrap();
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
                *t0 = time;
            }
            self.reading.store(value, Ordering::Relaxed);
        } else {
            self.reading.store(value, Ordering::Relaxed);
            self.empty.store(false, Ordering::Relaxed);
            let mut t0 = self.refreshed.write().unwrap();
            *t0 = time;
        }
    }

    /// Increment a counter by an amount
    pub fn increment_counter(&self, value: <Value as Atomic>::Primitive) {
        self.empty.store(false, Ordering::Relaxed);
        self.reading.fetch_add(value, Ordering::Relaxed);
    }

    /// Updates a gauge reading if the new value is newer than the stored value.
    pub fn record_gauge(&self, time: Instant, value: <Value as Atomic>::Primitive) {
        {
            let t0 = self.refreshed.read().unwrap();
            if time <= *t0 {
                return;
            }
        }
        if let Some(summary) = &self.summary {
            summary.increment(time, value, 1_u8.into());
        }
        self.reading.store(value, Ordering::Relaxed);
        self.empty.store(false, Ordering::Relaxed);
        let mut t0 = self.refreshed.write().unwrap();
        *t0 = time;
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

    /// Set a summary to be used for an existing channel
    pub fn set_summary(&mut self, summary: Summary<Value, Count>) {
        let summary = summary.build();
        self.summary = Some(summary);
    }

    /// Set a summary to be used for an existing channel
    pub fn add_summary(&mut self, summary: Summary<Value, Count>) {
        if self.summary.is_none() {
            self.set_summary(summary);
        }
    }
}
