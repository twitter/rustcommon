use crate::Metric;
use std::{
    any::Any,
    sync::atomic::{AtomicI64, Ordering},
};

/// A gauge. Indicates the current value of some host parameter.
///
/// # Example
/// ```
/// # use metrics_v2::{metric, Gauge};
/// metric! {
///     #[name = "my.gauge"]
///     static A_METHOD_RUNNING: Gauge = Gauge::new();
/// }
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
    pub const fn new() -> Self {
        Self::with_value(0)
    }

    pub const fn with_value(value: i64) -> Self {
        Self(AtomicI64::new(value))
    }

    #[inline]
    pub fn increment(&self) {
        self.add(1);
    }

    #[inline]
    pub fn decrement(&self) {
        self.sub(1);
    }

    #[inline]
    pub fn add(&self, value: i64) {
        self.0.fetch_add(value, Ordering::Relaxed);
    }

    #[inline]
    pub fn sub(&self, value: i64) {
        self.0.fetch_sub(value, Ordering::Relaxed);
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
