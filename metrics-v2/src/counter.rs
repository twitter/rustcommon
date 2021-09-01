use crate::Metric;
use std::{
    any::Any,
    sync::atomic::{AtomicU64, Ordering},
};

/// A counter. Can be incremented or added to.
///
/// # Example
/// ```
/// # use rustcommon_metrics_v2::{metric, Counter};
/// metric! {
///     #[name = "my.custom.metric"]
///     static MY_COUNTER: Counter = Counter::new();
/// }
///
/// fn a_method() {
///     MY_COUNTER.increment();
///     // ...
/// }
/// # a_method();
/// ```
#[derive(Default, Debug)]
pub struct Counter(AtomicU64);

impl Counter {
    /// Create a counter initialized to 0.
    pub const fn new() -> Self {
        Self::with_value(0)
    }

    /// Create a counter initialized to `value`.
    pub const fn with_value(value: u64) -> Self {
        Self(AtomicU64::new(value))
    }

    #[inline]
    pub fn increment(&self) -> u64 {
        self.add(1)
    }

    #[inline]
    pub fn add(&self, value: u64) -> u64 {
        self.0.fetch_add(value, Ordering::Relaxed)
    }

    #[inline]
    pub fn value(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn set(&self, value: u64) -> u64 {
        self.0.swap(value, Ordering::Relaxed)
    }

    #[inline]
    pub fn reset(&self) -> u64 {
        self.set(0)
    }
}

impl Metric for Counter {
    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}
