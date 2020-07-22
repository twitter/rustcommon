// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::source::Source;

/// Statistics define what a given channel is tracking. The statistic must have
/// a name and a source. There are additional methods which may return a value
/// and have default implementations returning `None`. These additional fields
/// may be used to specify the scope for a given statistic or other metadata.
pub trait Statistic {
    /// The name, which is used in the standard exposition format, eg:
    /// `collection/scope/name/reading: value`
    fn name(&self) -> &str;

    /// An optional collection, which is used in the standard exposition format,
    /// eg: `collection/scope/name/reading: value`
    fn collection(&self) -> Option<&str> {
        None
    }

    /// An optional scope, which is used in the standard exposition format, eg:
    /// `collection/scope/name/reading: value`
    fn scope(&self) -> Option<&str> {
        None
    }

    /// the unit of measurement
    fn unit(&self) -> Option<&str> {
        None
    }

    /// describe the meaning of the statistic
    fn description(&self) -> Option<&str> {
        None
    }

    /// the source of the measurement, such as a counter or gauge
    fn source(&self) -> Source;
}
