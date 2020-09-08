// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::{Channel, Primitive, Source, Statistic};

use dashmap::DashMap;
use rustcommon_atomics::*;

use std::time::Instant;

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
