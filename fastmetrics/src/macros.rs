// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[macro_export(local_inner_macros)]
/// Get the value for a counter.
/// See [`Metrics::get_counter()`](struct.Metrics.html#method.get_counter)
macro_rules! get_counter {
    ($($arg:tt)+) => (
        $crate::_metrics().get_counter($($arg)+)
    )
}

#[macro_export(local_inner_macros)]
/// Set a value for a counter, overwriting the previous value.
/// See [`Metrics::set_counter()`](struct.Metrics.html#method.set_counter)
macro_rules! set_counter {
    ($($arg:tt)+) => (
        $crate::_metrics().set_counter($($arg)+)
    )
}

#[macro_export(local_inner_macros)]
/// Increment a counter's value by one.
/// See [`Metrics::increment_counter()`](struct.Metrics.html#method.increment_counter_by)
macro_rules! increment_counter {
    ($($arg:tt)+) => (
        $crate::_metrics().increment_counter_by($($arg)+, 1)
    )
}

#[macro_export(local_inner_macros)]
/// Increment a counter's value by `N`.
/// See [`Metrics::increment_counter()`](struct.Metrics.html#method.increment_counter_by)
macro_rules! increment_counter_by {
    ($($arg:tt)+) => (
        $crate::_metrics().increment_counter_by($($arg)+)
    )
}

#[macro_export(local_inner_macros)]
/// Get the value for a gauge.
/// See [`Metrics::get_gauge()`](struct.Metrics.html#method.get_gauge)
macro_rules! get_gauge {
    ($($arg:tt)+) => (
        $crate::_metrics().get_gauge($($arg)+)
    )
}

#[macro_export(local_inner_macros)]
/// Set the value for a gauge, overwriting the previous value.
/// See [`Metrics::set_gauge()`](struct.Metrics.html#method.set_gauge)
macro_rules! set_gauge {
    ($($arg:tt)+) => (
        $crate::_metrics().set_gauge($($arg)+)
    )
}

#[macro_export(local_inner_macros)]
/// Increment a gauge's value by one.
/// See [`Metrics::increment_gauge()`](struct.Metrics.html#method.increment_gauge_by)
macro_rules! increment_gauge {
    ($($arg:tt)+) => (
        $crate::_metrics().increment_gauge_by($($arg)+, 1)
    )
}

#[macro_export(local_inner_macros)]
/// Increment a gauge's value by `N`.
/// See [`Metrics::increment_gauge()`](struct.Metrics.html#method.increment_gauge_by)
macro_rules! increment_gauge_by {
    ($($arg:tt)+) => (
        $crate::_metrics().increment_gauge_by($($arg)+)
    )
}

#[macro_export(local_inner_macros)]
/// Decrement a gauge's value by one.
/// See [`Metrics::decrement_gauge()`](struct.Metrics.html#method.decrement_gauge_by)
macro_rules! decrement_gauge {
    ($($arg:tt)+) => (
        $crate::_metrics().decrement_gauge_by($($arg)+, 1)
    )
}

#[macro_export(local_inner_macros)]
/// Decrement a gauge's value by `N`.
/// See [`Metrics::decrement_gauge()`](struct.Metrics.html#method.decrement_gauge_by)
macro_rules! decrement_gauge_by {
    ($($arg:tt)+) => (
        $crate::_metrics().decrement_gauge_by($($arg)+)
    )
}
