// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use parking_lot::{Mutex, MutexGuard};
use std::mem::ManuallyDrop;
use std::pin::Pin;

use rustcommon_metrics::*;

// All tests manipulate global state. Need a mutex to ensure test execution
// doesn't overlap.
static TEST_MUTEX: Mutex<()> = parking_lot::const_mutex(());

/// RAII guard that ensures
/// - All dynamic metrics are removed after each test
/// - No two tests run concurrently
struct TestGuard {
    _lock: MutexGuard<'static, ()>,
}

impl TestGuard {
    pub fn new() -> Self {
        Self {
            _lock: TEST_MUTEX.lock(),
        }
    }
}

impl Drop for TestGuard {
    fn drop(&mut self) {
        let to_unregister = metrics()
            .dynamic_metrics()
            .iter()
            .map(|entry| entry.metric() as *const dyn Metric)
            .collect::<Vec<_>>();

        for metric in to_unregister {
            dynmetrics::unregister(metric);
        }
    }
}

#[test]
fn register_unregister() {
    let _guard = TestGuard::new();

    let metric = Counter::new();
    let entry = unsafe { MetricEntry::new_unchecked(&metric, "register_unregister".into()) };

    dynmetrics::register(entry);

    assert_eq!(metrics().dynamic_metrics().len(), 1);
    dynmetrics::unregister(&metric);
    assert_eq!(metrics().dynamic_metrics().len(), 0);
}

#[test]
fn wrapped_register_unregister() {
    let _guard = TestGuard::new();

    let metric = DynBoxedMetric::new(Counter::new(), "wrapped_register_unregister");

    assert_eq!(metrics().dynamic_metrics().len(), 1);
    drop(metric);
    assert_eq!(metrics().dynamic_metrics().len(), 0);
}

#[test]
fn pinned_register_unregister() {
    let _guard = TestGuard::new();

    let mut metric_ = ManuallyDrop::new(DynPinnedMetric::new(Counter::new()));
    let metric = unsafe { Pin::new_unchecked(&*metric_) };
    metric.register("pinned_register_unregister");

    assert_eq!(metrics().dynamic_metrics().len(), 1);
    unsafe { ManuallyDrop::drop(&mut metric_) };
    assert_eq!(metrics().dynamic_metrics().len(), 0);
}

#[test]
fn pinned_scope() {
    let _guard = TestGuard::new();

    {
        let metric = DynPinnedMetric::new(Counter::new());
        let metric = unsafe { Pin::new_unchecked(&metric) };
        metric.register("pinned_scope");

        assert_eq!(metrics().dynamic_metrics().len(), 1);
    }
    assert_eq!(metrics().dynamic_metrics().len(), 0);
}

#[test]
fn pinned_dup_register() {
    let _guard = TestGuard::new();

    {
        let metric = DynPinnedMetric::new(Counter::new());
        let metric = unsafe { Pin::new_unchecked(&metric) };
        metric.register("pinned_dup_1");
        metric.register("pinned_dup_2");

        assert_eq!(metrics().dynamic_metrics().len(), 2);
    }
    assert_eq!(metrics().dynamic_metrics().len(), 0);
}

#[test]
fn multi_metric() {
    let _guard = TestGuard::new();

    let m1 = DynBoxedMetric::new(Counter::new(), "multi_metric_1");
    let m2 = DynBoxedMetric::new(Counter::new(), "multi_metric_2");

    assert_eq!(metrics().dynamic_metrics().len(), 2);
    drop(m1);
    assert_eq!(metrics().dynamic_metrics().len(), 1);
    drop(m2);
    assert_eq!(metrics().dynamic_metrics().len(), 0);
}
