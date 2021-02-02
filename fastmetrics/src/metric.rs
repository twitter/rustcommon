// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::fmt::Display;

/// The `Metric` trait is used to constrain valid metric types.
pub trait Metric: Display {
    /// Return the [`Source`] of a `Metric`
    fn source(&self) -> Source;

    /// Return a unique `usize` index for a `Metric`. Collisions will result in
    /// unexpected behaviors. This can generally be provided by `Into<usize>`
    /// for the concrete type.
    fn index(&self) -> usize;
}

#[derive(PartialEq, Copy, Clone)]
/// Defines `Metric` sources
pub enum Source {
    /// Counters are unsigned 64-bit monotonically increasing integer types.
    Counter,
    /// Gauges are signed 64-bit integer types.
    Gauge,
}

impl std::fmt::Display for Source {
    fn fmt(&self, w: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Source::Counter => write!(w, "counter"),
            Source::Gauge => write!(w, "gauge"),
        }
    }
}

impl std::fmt::Debug for Source {
    fn fmt(&self, w: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Source::Counter => write!(w, "counter"),
            Source::Gauge => write!(w, "gauge"),
        }
    }
}
