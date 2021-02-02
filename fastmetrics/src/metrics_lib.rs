// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

pub trait MetricsLib {
    // counters
    fn get_counter(&self, metric: &dyn Metric) -> Result<u64, MetricsError>;
    fn increment_counter_by(&self, metric: &dyn Metric, value: u64);
    fn set_counter(&self, metric: &dyn Metric, value: u64);

    // gauges
    fn get_gauge(&self, metric: &dyn Metric) -> Result<i64, MetricsError>;
    fn set_gauge(&self, metric: &dyn Metric, value: i64);
    fn increment_gauge_by(&self, metric: &dyn Metric, value: i64);
    fn decrement_gauge_by(&self, metric: &dyn Metric, value: i64);
}

impl<T> MetricsLib for std::boxed::Box<T>
where
    T: ?Sized + MetricsLib,
{
    fn set_counter(&self, metric: &dyn Metric, value: u64) {
        self.as_ref().set_counter(metric, value)
    }

    fn get_counter(&self, metric: &dyn Metric) -> Result<u64, MetricsError> {
        self.as_ref().get_counter(metric)
    }

    fn increment_counter_by(&self, metric: &dyn Metric, value: u64) {
        self.as_ref().increment_counter_by(metric, value)
    }

    fn get_gauge(&self, metric: &dyn Metric) -> Result<i64, MetricsError> {
        self.as_ref().get_gauge(metric)
    }

    fn set_gauge(&self, metric: &dyn Metric, value: i64) {
        self.as_ref().set_gauge(metric, value)
    }

    fn increment_gauge_by(&self, metric: &dyn Metric, value: i64) {
        self.as_ref().increment_gauge_by(metric, value)
    }

    fn decrement_gauge_by(&self, metric: &dyn Metric, value: i64) {
        self.as_ref().decrement_gauge_by(metric, value)
    }
}
