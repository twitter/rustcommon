// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Easily registered distributed metrics.
//!
//! More docs todo...
//!
//! # Creating a Metric
//! Registering a metric is straightforward. All that's needed is to declare a
//! static within the [`metric!`] macro. By default, the metric will have the
//! name of the path to the static variable you used to declare it but this can
//! be overridden by adding a `#[name]` attribute.
//!
//! ```
//! # use rustcommon_metrics_v2::*;
//! metric! {
//!     /// A counter metric named "<crate name>::COUNTER_A"
//!     static COUNTER_A: Counter = Counter::new();
//!
//!     /// A counter metric named "my.metric.name"
//!     #[name = "my.metric.name"]
//!     static COUNTER_B: Counter = Counter::new();
//! }
//! ```
//!
//! # Accessing Metrics
//! All metrics registered via the [`metric!`] macro can be accessed by calling
//! the [`metrics`] function. This will return a slice with one [`MetricEntry`]
//! instance per metric that was registered via the [`metric!`] macro.
//!
//! Suppose we have the metrics declared in the example above.
//! ```
//! # // This should remain in sync with the example above.
//! # use rustcommon_metrics_v2::*;
//! # metric! {
//! #     /// A counter metric named "COUNTER_A"
//! #     static COUNTER_A: Counter = Counter::new();
//! #
//! #     /// A counter metric named "my.metric.name"
//! #     #[name = "my.metric.name"]
//! #     static COUNTER_B: Counter = Counter::new();
//! # }
//! let metrics = metrics();
//! // Metrics may be in any arbitrary order
//! let mut names: Vec<_> = metrics.iter().map(|metric| metric.name()).collect();
//! names.sort();
//!
//! assert_eq!(names[0], "my.metric.name");
//! assert_eq!(names[1], concat!(module_path!(), "::", "COUNTER_A"));
//! ```
//!
//! # How it Works
//! Behind the scenes, this crate uses the [`linkme`] crate to create a
//! distributed slice containing a [`MetricEntry`] instance for each metric that
//! is registered via the [`metric!`] macro.

use std::any::Any;

mod counter;
mod gauge;
mod macros;

pub use crate::counter::Counter;
pub use crate::gauge::Gauge;

#[doc(hidden)]
pub mod export {
    pub extern crate linkme;

    #[linkme::distributed_slice]
    pub static METRICS: [crate::MetricEntry] = [..];
}

/// The list of all metrics registered via the [`metric!`] macro.
///
/// Names within metrics are not guaranteed to be unique and no aggregation of
/// metrics with the same name is done.
pub fn metrics() -> &'static [MetricEntry] {
    &*crate::export::METRICS
}

/// Global interface to a metric.
///
/// Most use of metrics should use the directly declared constants.
pub trait Metric: Sync {
    /// Indicate whether this metric has been set up.
    ///
    /// Generally, if this returns `false` then the other methods on this
    /// trait should return `None`.
    fn is_enabled(&self) -> bool {
        true
    }

    /// Get the current metric as an [`Any`] instance. This is meant to allow
    /// custom processing for known metric types.
    ///
    /// [`Any`]: std::any::Any
    fn as_any(&self) -> Option<&dyn Any>;
}

/// A statically declared metric entry.
pub struct MetricEntry {
    // These fields need to be public until it is possibe to create a const method with 
    // &'static dyn Metric as a parameter.
    #[doc(hidden)]
    pub metric: &'static dyn Metric,
    #[doc(hidden)]
    pub name: &'static str,
}

impl MetricEntry {
    /// Get a reference to the metric that this entry corresponds to.
    pub fn metric(&self) -> &'static dyn Metric {
        self.metric
    }

    /// Get the name of this metric.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl std::ops::Deref for MetricEntry {
    type Target = dyn Metric;

    fn deref(&self) -> &Self::Target {
        self.metric()
    }
}

impl std::fmt::Debug for MetricEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricEntry")
            .field("name", &self.name())
            .field("metric", &"<dyn Metric>")
            .finish()
    }
}
