// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Easily registered distributed metrics.
//!
//! More docs todo...
//!
//! # Creating a Metric
//! Registering a metric is straightforward. All that's needed is to declare a
//! static within the [`metric`] macro. By default, the metric will have the
//! name of the path to the static variable you used to declare it but this can
//! be overridden by passing the `name` parameter to the macro.
//!
//! ```
//! # // This should remain in sync with the example below.
//! use rustcommon_metrics::*;
//! /// A counter metric named "<crate name>::COUNTER_A"
//! #[metric]
//! static COUNTER_A: Counter = Counter::new();
//!
//! /// A counter metric named "my.metric.name"
//! #[metric(name = "my.metric.name")]
//! static COUNTER_B: Counter = Counter::new();
//! #
//! # let metrics = metrics();
//! # // Metrics may be in any arbitrary order
//! # let mut names: Vec<_> = metrics.iter().map(|metric| metric.name()).collect();
//! # names.sort();
//! #
//! # assert_eq!(names.len(), 2);
//! # assert_eq!(names[0], "my.metric.name");
//! # assert_eq!(names[1], concat!(module_path!(), "::", "COUNTER_A"));
//! ```
//!
//! # Accessing Metrics
//! All metrics registered via the [`metric`] macro can be accessed by calling
//! the [`metrics`] function. This will return an instance of the [`Metric`]
//! struct which allows you to access all staticly and dynamically registered
//! metrics.
//!
//! Suppose we have the metrics declared in the example above.
//! ```
//! # // This should remain in sync with the example above.
//! # use rustcommon_metrics::*;
//! # /// A counter metric named "<crate name>::COUNTER_A"
//! # #[metric]
//! # static COUNTER_A: Counter = Counter::new();
//! #
//! # /// A counter metric named "my.metric.name"
//! # #[metric(name = "my.metric.name")]
//! # static COUNTER_B: Counter = Counter::new();
//! #
//! let metrics = metrics();
//! // Metrics may be in any arbitrary order
//! let mut names: Vec<_> = metrics.iter().map(|metric| metric.name()).collect();
//! names.sort();
//!
//! assert_eq!(names.len(), 2);
//! assert_eq!(names[0], "my.metric.name");
//! assert_eq!(names[1], concat!(module_path!(), "::", "COUNTER_A"));
//! ```
//!
//! # How it Works
//! Behind the scenes, this crate uses the [`linkme`] crate to create a
//! distributed slice containing a [`MetricEntry`] instance for each metric that
//! is registered via the [`metric`] attribute.

use parking_lot::RwLockReadGuard;
use std::any::Any;
use std::borrow::Cow;

mod counter;
mod gauge;
mod heatmap;
mod lazy;

extern crate self as rustcommon_metrics;

pub mod dynmetrics;

pub use crate::counter::Counter;
pub use crate::dynmetrics::{DynBoxedMetric, DynPinnedMetric};
pub use crate::gauge::Gauge;
pub use crate::heatmap::Heatmap;
pub use crate::lazy::{Lazy, Relaxed};

pub use rustcommon_metrics_derive::{metric, to_lowercase};
pub use rustcommon_time::*;

#[doc(hidden)]
pub mod export {
    pub extern crate linkme;

    #[linkme::distributed_slice]
    pub static METRICS: [crate::MetricEntry] = [..];
}

#[macro_export]
#[rustfmt::skip]
macro_rules! counter {
    ($name:ident) => {
        #[metric(
            name = rustcommon_metrics::to_lowercase!($name)
        )]
        pub static $name: Counter = Counter::new();
    };
    ($name:ident, $description:tt) => {
        #[metric(
            name = rustcommon_metrics::to_lowercase!($name),
            description = $description
        )]
        pub static $name: Counter = Counter::new();
    };
}

#[macro_export]
#[rustfmt::skip]
macro_rules! gauge {
    ($name:ident) => {
        #[metric(
            name = rustcommon_metrics::to_lowercase!($name)
        )]
        pub static $name: Gauge = Gauge::new();
    };
    ($name:ident, $description:tt) => {
        #[metric(
            name = rustcommon_metrics::to_lowercase!($name),
            description = $description
        )]
        pub static $name: Gauge = Gauge::new();
    };
}

#[macro_export]
#[rustfmt::skip]
macro_rules! heatmap {
    ($name:ident, $max:expr) => {
        #[metric(
            name = rustcommon_metrics::to_lowercase!($name)
        )]
        pub static $name: Relaxed<Heatmap> = Relaxed::new(|| {
            Heatmap::builder()
                .maximum_value($max as _)
                .min_resolution(1)
                .min_resolution_range(1024)
                .span(rustcommon_metrics::Duration::<rustcommon_metrics::Nanoseconds<u64>>::from_secs(60))
                .resolution(rustcommon_metrics::Duration::<rustcommon_metrics::Nanoseconds<u64>>::from_secs(1))
                .build()
                .expect("bad heatmap configuration")
        });
    };
    ($name:ident, $max:expr, $description:tt) => {
        #[metric(
            name = rustcommon_metrics::to_lowercase!($name),
            description = $description
        )]
        pub static $name: Relaxed<Heatmap> = Relaxed::new(|| {
            Heatmap::builder()
                .maximum_value($max as _)
                .min_resolution(1)
                .min_resolution_range(1024)
                .span(rustcommon_metrics::Duration::<rustcommon_metrics::Nanoseconds<u64>>::from_secs(60))
                .resolution(rustcommon_metrics::Duration::<rustcommon_metrics::Nanoseconds<u64>>::from_secs(1))
                .build()
                .expect("bad heatmap configuration")
        });
    };
}

/// Global interface to a metric.
///
/// Most use of metrics should use the directly declared constants.
pub trait Metric: Send + Sync + 'static {
    /// Indicate whether this metric has been set up.
    ///
    /// Generally, if this returns `false` then the other methods on this
    /// trait should return `None`.
    fn is_enabled(&self) -> bool {
        self.as_any().is_some()
    }

    /// Get the current metric as an [`Any`] instance. This is meant to allow
    /// custom processing for known metric types.
    ///
    /// [`Any`]: std::any::Any
    fn as_any(&self) -> Option<&dyn Any>;
}

/// A statically declared metric entry.
pub struct MetricEntry {
    metric: MetricWrapper,
    name: Cow<'static, str>,
    namespace: Option<&'static str>,
    description: Option<&'static str>,
}

impl MetricEntry {
    #[doc(hidden)]
    pub const fn _new_const(
        metric: MetricWrapper,
        name: &'static str,
        namespace: &'static str,
        description: &'static str,
    ) -> Self {
        let namespace = if namespace.is_empty() {
            None
        } else {
            Some(namespace)
        };
        let description = if description.is_empty() {
            None
        } else {
            Some(description)
        };
        Self {
            metric,
            name: Cow::Borrowed(name),
            namespace,
            description,
        }
    }

    /// Create a new metric entry with the provided metric and name.
    pub fn new(metric: &'static dyn Metric, name: Cow<'static, str>) -> Self {
        // SAFETY: The lifetimes here are static.
        unsafe { Self::new_unchecked(metric, name) }
    }

    /// Create a new metric entry with the provided metric and name.
    ///
    /// # Safety
    /// This method is only safe to call if
    /// - `metric` points to a valid `dyn Metric` instance, and,
    /// - the metric instance outlives this `MetricEntry`.
    pub unsafe fn new_unchecked(metric: *const dyn Metric, name: Cow<'static, str>) -> Self {
        Self {
            metric: MetricWrapper(metric),
            name,
            namespace: None,
            description: None,
        }
    }

    /// Get a reference to the metric that this entry corresponds to.
    pub fn metric(&self) -> &dyn Metric {
        unsafe { &*self.metric.0 }
    }

    /// Get the name of this metric.
    pub fn name(&self) -> &str {
        &*self.name
    }

    /// Get the namespace of this metric.
    pub fn namespace(&self) -> Option<&str> {
        self.namespace
    }

    /// Get the description of this metric.
    pub fn description(&self) -> Option<&str> {
        self.description
    }
}

unsafe impl Send for MetricEntry {}
unsafe impl Sync for MetricEntry {}

impl std::ops::Deref for MetricEntry {
    type Target = dyn Metric;

    #[inline]
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

/// You can't use `dyn <trait>s` directly in const methods for now but a wrapper
/// is fine. This wrapper is a work around to allow us to use const constructors
/// for the MetricEntry struct.
#[doc(hidden)]
pub struct MetricWrapper(pub *const dyn Metric);

/// The list of all metrics registered via the either [`metric`] attribute or by
/// using the types within the [`dynmetrics`] module.
///
/// Names within metrics are not guaranteed to be unique and no aggregation of
/// metrics with the same name is done.
pub fn metrics() -> Metrics {
    Metrics {
        dyn_metrics: crate::dynmetrics::get_registry(),
    }
}

/// Provides access to all registered metrics both static and dynamic.
///
/// **IMPORTANT:** Note that while any instance of this struct is live
/// attempting to register or unregister any dynamic metrics will block.
/// If this is done on the same thread as is currently working with an instance
/// of `Metrics` then it will cause a deadlock. If your application will be
/// registering and unregistering dynamic metrics then you should avoid holding
/// on to `Metrics` instances for long periods of time.
///
/// `Metrics` instances can be created via the [`metrics`] function.
pub struct Metrics {
    dyn_metrics: RwLockReadGuard<'static, dynmetrics::DynMetricsRegistry>,
}

impl Metrics {
    /// A list containing all metrics that were registered via the [`metric`]
    /// attribute macro.
    pub fn static_metrics(&self) -> &'static [MetricEntry] {
        &*crate::export::METRICS
    }

    /// A list containing all metrics that were dynamically registered.
    pub fn dynamic_metrics(&self) -> &[MetricEntry] {
        self.dyn_metrics.metrics()
    }

    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a Metrics {
    type Item = &'a MetricEntry;

    type IntoIter =
        std::iter::Chain<std::slice::Iter<'a, MetricEntry>, std::slice::Iter<'a, MetricEntry>>;

    fn into_iter(self) -> Self::IntoIter {
        self.static_metrics()
            .iter()
            .chain(self.dynamic_metrics().iter())
    }
}

/// The type of the static generated by `#[metric]`.
///
/// This exports the name of the generated metric so that other code
/// can use it.
pub struct MetricInstance<M> {
    // The generated code by the #[metric] attribute needs to access this
    // directly so it needs to be public.
    #[doc(hidden)]
    pub metric: M,
    name: &'static str,
    description: Option<&'static str>,
}

impl<M> MetricInstance<M> {
    #[doc(hidden)]
    pub const fn new(metric: M, name: &'static str, description: &'static str) -> Self {
        let description = if description.is_empty() {
            None
        } else {
            Some(description)
        };
        Self {
            metric,
            name,
            description,
        }
    }

    /// Get the name of this metric.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Get the description of this metric.
    pub const fn description(&self) -> Option<&'static str> {
        self.description
    }
}

impl<M> std::ops::Deref for MetricInstance<M> {
    type Target = M;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.metric
    }
}

impl<M> std::ops::DerefMut for MetricInstance<M> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.metric
    }
}

impl<M> AsRef<M> for MetricInstance<M> {
    #[inline]
    fn as_ref(&self) -> &M {
        &self.metric
    }
}

impl<M> AsMut<M> for MetricInstance<M> {
    #[inline]
    fn as_mut(&mut self) -> &mut M {
        &mut self.metric
    }
}
