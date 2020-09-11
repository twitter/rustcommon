// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::entry::Entry;
use crate::*;
use dashmap::{DashMap, DashSet};
use rustcommon_atomics::Atomic;

pub struct Outputs<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    pub(crate) outputs: DashMap<Entry<Value, Count>, DashSet<ApproxOutput>>,
}

impl<Value, Count> Outputs<Value, Count>
where
    Value: crate::Value,
    Count: crate::Count,
    <Value as Atomic>::Primitive: Primitive,
    <Count as Atomic>::Primitive: Primitive,
    u64: From<<Value as Atomic>::Primitive> + From<<Count as Atomic>::Primitive>,
{
    pub fn new() -> Self {
        Self {
            outputs: DashMap::new(),
        }
    }

    pub fn register(&self, statistic: &dyn Statistic<Value, Count>, output: Output) {
        let entry = Entry::from(statistic);
        let output = ApproxOutput::from(output);
        if let Some(outputs) = self.outputs.get(&entry) {
            outputs.insert(output);
        } else {
            let outputs = DashSet::new();
            outputs.insert(output);
            self.outputs.insert(entry, outputs);
        }
    }

    pub fn deregister(&self, statistic: &dyn Statistic<Value, Count>, output: Output) {
        let entry = Entry::from(statistic);
        let output = ApproxOutput::from(output);
        if let Some(outputs) = self.outputs.get(&entry) {
            outputs.remove(&output);
        }
    }

    pub fn deregister_statistic(&self, statistic: &dyn Statistic<Value, Count>) {
        let entry = Entry::from(statistic);
        self.outputs.remove(&entry);
    }

    pub fn clear(&self) {
        self.outputs.clear()
    }
}

// Internal representation which approximates the percentile
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum ApproxOutput {
    Reading,
    Percentile(u64),
}

/// Defines an output that should be reported in a snapshot for a statistic
#[derive(Copy, Clone)]
pub enum Output {
    /// A counter or gauge reading
    Reading,
    /// A percentile from a statistic summary
    Percentile(f64),
}

impl From<Output> for ApproxOutput {
    fn from(output: Output) -> Self {
        match output {
            Output::Reading => Self::Reading,
            Output::Percentile(percentile) => {
                Self::Percentile((percentile * 1000000.0).ceil() as u64)
            }
        }
    }
}

impl From<ApproxOutput> for Output {
    fn from(output: ApproxOutput) -> Self {
        match output {
            ApproxOutput::Reading => Self::Reading,
            ApproxOutput::Percentile(percentile) => Self::Percentile(percentile as f64 / 1000000.0),
        }
    }
}
