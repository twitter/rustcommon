// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Metric;
use std::any::Any;
use std::sync::atomic::{AtomicI64, Ordering};

/// A gauge. Indicates the current value of some host parameter.
///
/// In case of overflow/underflow the gauge will wrap around. However,
/// internally it uses a signed 64-bit integer so for most use cases this should
/// be unlikely.
///
/// # Example
/// ```
/// # use rustcommon_metrics_v2::{metric, Gauge};
/// #[metric(name = "my.gauge")]
/// static A_METHOD_RUNNING: Gauge = Gauge::new();
///
/// fn a_method() {
///     A_METHOD_RUNNING.increment();
///     // ...
///     A_METHOD_RUNNING.decrement();
/// }
/// # a_method();
/// ```
#[derive(Default, Debug)]
pub struct Gauge(AtomicI64);

impl Gauge {
    /// Create a new guage with the default value of 0.
    pub const fn new() -> Self {
        Self::with_value(0)
    }

    /// Create a new guage with the provided initial value.
    pub const fn with_value(value: i64) -> Self {
        Self(AtomicI64::new(value))
    }

    /// Increment the value of this gauge by 1.
    /// 
    /// Returns the old value of the gauge.
    #[inline]
    pub fn increment(&self) -> i64 {
        self.add(1)
    }

    /// Decrement the value of this gauge by 1.
    /// 
    /// Returns the old value of the gauge.
    #[inline]
    pub fn decrement(&self) -> i64 {
        self.sub(1)
    }

    /// Increase the value of this gauge by `value`.
    /// 
    /// Returns the od value of the gauge.
    #[inline]
    pub fn add(&self, value: i64) -> i64 {
        self.0.fetch_add(value, Ordering::Relaxed)
    }

    /// Decrease the value of this gauge by `value`.
    /// 
    /// Returns the od value of the gauge.
    #[inline]
    pub fn sub(&self, value: i64) -> i64 {
        self.0.fetch_sub(value, Ordering::Relaxed)
    }

    #[inline]
    pub fn value(&self) -> i64 {
        self.0.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn set(&self, value: i64) -> i64 {
        self.0.swap(value, Ordering::Relaxed)
    }

    #[inline]
    pub fn reset(&self) -> i64 {
        self.set(0)
    }
}

impl Metric for Gauge {
    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}
