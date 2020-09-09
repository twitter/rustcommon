// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::{Atomic, Primitive, Source, Summary};

pub trait Statistic<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    /// The name is used to lookup the channel for the statistic and should be
    /// unique for each statistic.
    fn name(&self) -> &str;
    /// Indicates which source type the statistic tracks.
    fn source(&self) -> Source;
    /// Optionally, specify a summary type which is used to produce percentiles.
    fn summary(&self) -> Option<Summary<Value, Count>> {
        None
    }
}
