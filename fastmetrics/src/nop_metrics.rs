// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

pub struct NopMetrics;

impl MetricsLib for NopMetrics {
    fn set_counter(&self, _: &dyn Metric, _: u64) {}
    fn get_counter(&self, _: &dyn Metric) -> Result<u64, MetricsError> {
        Err(MetricsError::Unitialized)
    }
    fn increment_counter_by(&self, _: &dyn Metric, _: u64) {}
    fn set_gauge(&self, _: &dyn Metric, _: i64) {}
    fn get_gauge(&self, _: &dyn Metric) -> std::result::Result<i64, MetricsError> {
        Err(MetricsError::Unitialized)
    }
    fn increment_gauge_by(&self, _: &dyn Metric, _: i64) {}
    fn decrement_gauge_by(&self, _: &dyn Metric, _: i64) {}
}
