// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Statistic;
use crate::*;
use chashmap::CHashMap;
use std::collections::HashMap;
use std::sync::Arc;

/// The general structure which holds data and is used to add channels and their
/// outputs, record measurements, and produce readings
pub struct Metrics<T: 'static>
where
    T: Unsigned + SaturatingArithmetic + Default + FetchCompareStore,
    <T as Atomic>::Primitive: Default + PartialEq + Copy + From<u8>,
    u64: From<<T as Atomic>::Primitive>,
{
    data: CHashMap<String, Arc<Channel<T>>>,
}

impl<T> Clone for Metrics<T>
where
    T: Unsigned + SaturatingArithmetic + Default + FetchCompareStore,
    <T as Atomic>::Primitive: Default + PartialEq + Copy + From<u8>,
    u64: From<<T as Atomic>::Primitive>,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<T> Metrics<T>
where
    T: Unsigned + SaturatingArithmetic + Default + FetchCompareStore,
    <T as Atomic>::Primitive: Default + PartialEq + Copy + From<u8>,
    u64: From<<T as Atomic>::Primitive>,
{
    pub fn new() -> Self {
        Self {
            data: CHashMap::new(),
        }
    }

    pub fn record_counter(&self, statistic: &dyn Statistic, time: u64, value: u64) {
        if let Some(channel) = self.data.get(statistic.name()) {
            channel.record_counter(time, value);
        }
    }

    pub fn record_gauge(&self, statistic: &dyn Statistic, time: u64, value: u64) {
        if let Some(channel) = self.data.get(statistic.name()) {
            channel.record_gauge(time, value);
        }
    }

    pub fn record_distribution(
        &self,
        statistic: &dyn Statistic,
        time: u64,
        value: u64,
        count: <T as Atomic>::Primitive,
    ) {
        if let Some(channel) = self.data.get(statistic.name()) {
            channel.record_distribution(time, value, count);
        }
    }

    pub fn record_delta(&self, statistic: &dyn Statistic, time: u64, value: u64) {
        if let Some(channel) = self.data.get(statistic.name()) {
            channel.record_delta(time, value);
        }
    }

    pub fn record_increment(
        &self,
        statistic: &dyn Statistic,
        time: u64,
        count: <T as Atomic>::Primitive,
    ) {
        if let Some(channel) = self.data.get(statistic.name()) {
            channel.record_increment(time, count);
        }
    }

    pub fn record_time_interval(&self, statistic: &dyn Statistic, start: u64, stop: u64) {
        if let Some(channel) = self.data.get(statistic.name()) {
            channel.record_time_interval(start, stop);
        }
    }

    pub fn reading(&self, statistic: &dyn Statistic) -> Option<u64> {
        if let Some(channel) = self.data.get(statistic.name()) {
            Some(channel.reading())
        } else {
            None
        }
    }

    pub fn percentile(&self, statistic: &dyn Statistic, percentile: f64) -> Option<u64> {
        if let Some(channel) = self.data.get(statistic.name()) {
            channel.percentile(percentile)
        } else {
            None
        }
    }

    pub fn register(&self, statistic: &dyn Statistic, summary: Option<Summary>) {
        if !self.data.contains_key(statistic.name()) {
            let channel = Channel::new(statistic, summary);
            self.data
                .insert(statistic.name().to_string(), Arc::new(channel));
        }
    }

    pub fn deregister(&self, statistic: &dyn Statistic) {
        self.data.remove(statistic.name());
    }

    pub fn readings(&self) -> Vec<Reading> {
        let temp = self.data.clone();
        let mut result = Vec::new();
        for (_label, channel) in temp {
            result.extend(channel.readings());
        }
        result
    }

    pub fn hash_map(&self) -> HashMap<String, HashMap<Output, u64>> {
        let temp = self.data.clone();
        let mut result = HashMap::new();
        for (label, channel) in temp {
            result.insert(label.to_owned(), channel.hash_map());
        }
        result
    }

    #[cfg(feature = "waterfall")]
    pub fn save_files(&self) {
        for (_label, channel) in self.data.keys() {
            channel.save_files();
        }
    }

    pub fn add_output(&self, statistic: &dyn Statistic, output: Output) {
        if let Some(channel) = self.data.get(statistic.name()) {
            channel.add_output(output);
        }
    }

    pub fn delete_output(&self, statistic: &dyn Statistic, output: Output) {
        if let Some(channel) = self.data.get(statistic.name()) {
            channel.delete_output(output);
        }
    }

    pub fn latch(&self) {
        let temp = self.data.clone();
        for (label, _channel) in temp {
            if let Some(channel) = self.data.get(&label) {
                channel.latch();
            }
        }
    }

    pub fn zero(&self) {
        let temp = self.data.clone();
        for (label, _channel) in temp {
            if let Some(channel) = self.data.get(&label) {
                channel.zero();
            }
        }
    }

    pub fn clear(&self) {
        self.data.clear();
    }

    pub fn shrink_to_fit(&self) {
        self.data.shrink_to_fit();
    }
}

impl<T> Default for Metrics<T>
where
    T: Unsigned + SaturatingArithmetic + Default + FetchCompareStore,
    <T as Atomic>::Primitive: Default + PartialEq + Copy + From<u8>,
    u64: From<<T as Atomic>::Primitive>,
{
    fn default() -> Self {
        Self::new()
    }
}
