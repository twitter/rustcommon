// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

impl<T> MetricsLib for Metrics<T>
where
    T: Metric + Into<usize> + Copy,
{
    fn get_counter(&self, metric: &dyn Metric) -> Result<u64, MetricsError> {
        match self.get(metric.index()) {
            None => Err(MetricsError::NotRegistered(metric.to_string())),
            Some(Channel::Counter { counter, .. }) => Ok(counter.load(Ordering::Relaxed)),
            _ => Err(MetricsError::WrongType(
                metric.to_string(),
                metric.source(),
                Source::Counter,
            )),
        }
    }

    fn set_counter(&self, metric: &dyn Metric, value: u64) {
        if let Some(Channel::Counter { counter, .. }) = self.get(metric.index()) {
            counter.store(value, Ordering::Relaxed);
        }
    }

    fn increment_counter_by(&self, metric: &dyn Metric, value: u64) {
        if let Some(Channel::Counter { counter, .. }) = self.get(metric.index()) {
            counter.fetch_add(value, Ordering::Relaxed);
        }
    }

    fn get_gauge(&self, metric: &dyn Metric) -> Result<i64, MetricsError> {
        match self.get(metric.index()) {
            None => Err(MetricsError::NotRegistered(metric.to_string())),
            Some(Channel::Gauge { gauge, .. }) => Ok(gauge.load(Ordering::Relaxed)),
            _ => Err(MetricsError::WrongType(
                metric.to_string(),
                metric.source(),
                Source::Gauge,
            )),
        }
    }

    fn set_gauge(&self, metric: &dyn Metric, value: i64) {
        if let Some(Channel::Gauge { gauge, .. }) = self.get(metric.index()) {
            gauge.store(value, Ordering::Relaxed);
        }
    }

    fn increment_gauge_by(&self, metric: &dyn Metric, value: i64) {
        if let Some(Channel::Gauge { gauge, .. }) = self.get(metric.index()) {
            gauge.fetch_add(value, Ordering::Relaxed);
        }
    }

    fn decrement_gauge_by(&self, metric: &dyn Metric, value: i64) {
        if let Some(Channel::Gauge { gauge, .. }) = self.get(metric.index()) {
            gauge.fetch_sub(value, Ordering::Relaxed);
        }
    }
}

pub struct Metrics<T>
where
    T: Metric + Into<usize> + Copy,
{
    channels: Vec<Option<Channel<T>>>,
}

pub enum Channel<T>
where
    T: Metric,
{
    Counter { counter: AtomicU64, metric: T },
    Gauge { gauge: AtomicI64, metric: T },
}

impl<T> Metrics<T>
where
    T: Metric + Into<usize> + Copy,
{
    #[inline]
    fn get(&self, metric: usize) -> Option<&Channel<T>> {
        match self.channels.get(metric) {
            None | Some(None) => None,
            Some(c) => c.as_ref(),
        }
    }

    // NOTE: we redefine these functions here to improve the rustdocs since the
    // `MetricsLib` trait is private.

    /// Returns the value for the counter or some error.
    pub fn get_counter(&self, metric: &dyn Metric) -> Result<u64, MetricsError> {
        (self as &dyn MetricsLib).get_counter(metric)
    }

    /// Sets the counter's value, overwriting any previous value.
    pub fn set_counter(&self, metric: &dyn Metric, value: u64) {
        (self as &dyn MetricsLib).set_counter(metric, value)
    }

    /// Increments the counter's value by `n`.
    pub fn increment_counter_by(&self, metric: &dyn Metric, n: u64) {
        (self as &dyn MetricsLib).increment_counter_by(metric, n)
    }

    /// Returns the value for the gauge or some error.
    pub fn get_gauge(&self, metric: &dyn Metric) -> Result<i64, MetricsError> {
        (self as &dyn MetricsLib).get_gauge(metric)
    }

    /// Sets the gauge's value, overwriting any previous value.
    pub fn set_gauge(&self, metric: &dyn Metric, value: i64) {
        (self as &dyn MetricsLib).set_gauge(metric, value)
    }

    /// Increments the gauge's value by `n`.
    pub fn increment_gauge_by(&self, metric: &dyn Metric, n: i64) {
        (self as &dyn MetricsLib).increment_gauge_by(metric, n)
    }

    /// Decrements the gauge's value by `n`.
    pub fn decrement_gauge_by(&self, metric: &dyn Metric, n: i64) {
        (self as &dyn MetricsLib).decrement_gauge_by(metric, n)
    }
}

#[derive(Default)]
pub struct MetricsBuilder<T>
where
    T: Metric + Into<usize> + Copy,
{
    channels: Vec<Option<Channel<T>>>,
}

impl<'a, T: 'static + 'a> MetricsBuilder<T>
where
    T: Metric + Into<usize> + Copy,
{
    /// Create a new builder to configure the metrics library.
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
        }
    }

    /// Bulk registration of a collection of metrics.
    pub fn metrics<I>(mut self, metrics: I) -> Self
    where
        I: IntoIterator<Item = &'a T>,
    {
        for metric in metrics {
            self = self.metric(*metric);
        }

        self
    }

    /// Registration of a single metric.
    pub fn metric(mut self, metric: T) -> Self {
        let id: usize = metric.into();
        for _ in self.channels.len()..=id {
            self.channels.push(None);
        }
        match metric.source() {
            Source::Counter => {
                self.channels[id] = Some(Channel::Counter {
                    counter: Default::default(),
                    metric,
                });
            }
            Source::Gauge => {
                self.channels[id] = Some(Channel::Gauge {
                    gauge: Default::default(),
                    metric,
                });
            }
        }

        self
    }

    /// Builds the corresponding `Metrics` struct and sets it as the metrics
    /// library.
    pub fn build(self) -> Result<(), MetricsError> {
        let metrics = Metrics {
            channels: self.channels,
        };
        set_metrics(Box::new(metrics))
    }
}

// Based on Rust `log` crate: https://github.com/rust-lang/log
//
// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
fn set_metrics(metrics: Box<dyn MetricsLib>) -> Result<(), MetricsError> {
    let old_state = match STATE.compare_exchange(
        UNINITIALIZED,
        INITIALIZING,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(s) | Err(s) => s,
    };
    match old_state {
        UNINITIALIZED => {
            unsafe {
                METRICS = Box::leak(metrics);
            }
            STATE.store(INITIALIZED, Ordering::SeqCst);
            Ok(())
        }
        INITIALIZING => {
            while STATE.load(Ordering::SeqCst) == INITIALIZING {
                std::hint::spin_loop();
            }
            Err(MetricsError::AlreadyInitialized)
        }
        _ => Err(MetricsError::AlreadyInitialized),
    }
}
