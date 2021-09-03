//! Methods and structs for workin with dynamically created and destroyed
//! metrics.
//!
//! Generally users should not need to use anything in this module with the
//! exception of [`DynPinnedMetric`] and [`DynBoxedMetric`].

use std::{borrow::Cow, marker::PhantomPinned, ops::Deref, pin::Pin};

use crate::{Metric, MetricEntry};

// We use parking_lot here since it avoids lock poisioning
use parking_lot::{const_rwlock, RwLock, RwLockReadGuard};

pub(crate) struct DynMetricsRegistry {
    metrics: Vec<MetricEntry>,
}

impl DynMetricsRegistry {
    const fn new() -> Self {
        Self {
            metrics: Vec::new(),
        }
    }

    fn register(&mut self, entry: MetricEntry) {
        self.metrics.push(entry);
    }

    fn unregister(&mut self, metric: *const dyn Metric) {
        self.metrics
            .retain(|x| x.metric.0 as *const () != metric as *const ());
    }

    pub(crate) fn metrics(&self) -> &[MetricEntry] {
        &self.metrics
    }
}

static REGISTRY: RwLock<DynMetricsRegistry> = const_rwlock(DynMetricsRegistry::new());

pub(crate) fn get_registry() -> RwLockReadGuard<'static, DynMetricsRegistry> {
    REGISTRY.read()
}

/// Registers a new dynamic metric entry.
///
/// The [`MetricEntry`] instance will be kept until an [`unregister`] call is
/// made with a metric pointer that matches the one within the [`MetricEntry`].
/// When using this take care to note how it interacts with [`MetricEntry`]'s
/// safety guarantees.
pub fn register(entry: MetricEntry) {
    REGISTRY.write().register(entry);
}

/// Unregisters all dynamic entries added via [`register`] that point to the
/// same address as `metric`.
///
/// This function may remove multiple entries if the same metric has been
/// registered multiple times.
pub fn unregister(metric: *const dyn Metric) {
    REGISTRY.write().unregister(metric);
}

/// A dynamic metric that stores the metric inline.
///
/// This is a dynamic metric that relies on pinning guarantees to ensure that
/// the stored metric can be safely accessed from other threads looking through
/// the global dynamic metrics registry. As it requires pinning, it is somewhat
/// unweildy to use. Most use cases can probably use [`DynBoxedMetric`] instead.
///
/// To use this, first create the `DynPinnedMetric` and then, once it is pinned,
/// call [`register`] any number of times with all of the names the metric
/// should be registered under. When the `DynPinnedMetric` instance is dropped
/// it will unregister all the metric entries added via [`register`].
///
/// # Example
/// ```
/// # use rustcommon_metrics_v2::*;
/// # use std::pin::Pin;
/// let my_dyn_metric = DynPinnedMetric::new(Counter::new());
/// // Normally you would use some utility to do this. (e.g. pin-utils)
/// let my_dyn_metric = unsafe { Pin::new_unchecked(&my_dyn_metric) };
/// my_dyn_metric.register("a.dynamic.counter");
///
/// let metrics = metrics();
/// assert_eq!(metrics.dynamic_metrics()[0].name(), "a.dynamic.counter");
/// ```
///
/// [`register`]: crate::dynmetrics::DynPinnedMetric::register
pub struct DynPinnedMetric<M: Metric> {
    metric: M,
    // This type relies on Pin's guarantees for correctness. Allowing it to be unpinned would cause
    // errors.
    _marker: PhantomPinned,
}

impl<M: Metric> DynPinnedMetric<M> {
    /// Create a new `DynPinnedMetric` with the provided internal metric.
    ///
    /// This does not register the metric. To do that call [`register`].
    ///
    /// [`register`]: self::DynPinnedMetric::register
    pub fn new(metric: M) -> Self {
        Self {
            metric,
            _marker: PhantomPinned,
        }
    }

    /// Register this metric in the global list of dynamic metrics with `name`.
    ///
    /// Calling this multiple times will result in the same metric being
    /// registered multiple times under potentially different names.
    pub fn register(self: Pin<&Self>, name: impl Into<Cow<'static, str>>) {
        // SAFETY:
        // To prove that this is safe we need to list out a few guarantees/requirements:
        //  - Pin ensures that the memory of this struct instance will not be reused
        //    until the drop call completes.
        //  - MetricEntry::new_unchecked requires that the metric reference outlive
        //    created the MetricEntry instance.
        //
        // Finally, register, will keep the MetricEntry instance in a global list until
        // the corresponding unregister call is made.
        //
        // Taking all of these together, we can guarantee that self.metric will not be
        // dropped until this instance of DynPinnedMetric is dropped itself. At that
        // point, drop calls unregister which will drop the MetricEntry instance. This
        // ensures that the references to self.metric in REGISTRY will always be valid
        // and that this method is safe.
        unsafe { register(MetricEntry::new_unchecked(&self.metric, name.into())) };
    }
}

impl<M: Metric> Drop for DynPinnedMetric<M> {
    fn drop(&mut self) {
        // If this metric has not been registered then nothing will be removed.
        unregister(&self.metric);
    }
}

impl<M: Metric> Deref for DynPinnedMetric<M> {
    type Target = M;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.metric
    }
}

/// A dynamic metric that stores the metric instance on the heap.
///
/// This avoids a lot of the hangup with [`DynPinnedMetric`] as it allows for
/// moving the `DynBoxedMetric` without having to worry about pinning or safety
/// issues. However, this comes at the expense of requiring a heap allocation
/// for the metric.
///
/// # Example
/// ```
/// # use rustcommon_metrics_v2::*;
/// let my_gauge = DynBoxedMetric::new(Gauge::new(), "my.dynamic.gauge");
///
/// let metrics = metrics();
/// assert_eq!(metrics.dynamic_metrics()[0].name(), "my.dynamic.gauge");
/// ```
pub struct DynBoxedMetric<M: Metric> {
    metric: Pin<Box<DynPinnedMetric<M>>>,
}

impl<M: Metric> DynBoxedMetric<M> {
    /// Create a new dynamic metric using the provided metric type with the
    /// provided `name`.
    pub fn new(metric: M, name: impl Into<Cow<'static, str>>) -> Self {
        let metric = Box::pin(DynPinnedMetric::new(metric));
        metric.as_ref().register(name.into());

        Self { metric }
    }
}

impl<M: Metric> Deref for DynBoxedMetric<M> {
    type Target = M;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.metric
    }
}
