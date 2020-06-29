// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::time::Duration;
use rustcommon_datastructures::*;

#[derive(Clone, Copy)]
pub enum SummaryType {
    Histogram(u64, u32, Option<Duration>),
}

#[derive(Clone, Copy)]
pub struct Summary {
    inner: SummaryType,
}

impl Summary {
    pub fn histogram(max: u64, precision: u32, window: Option<Duration>) -> Self {
        Self {
            inner: SummaryType::Histogram(max, precision, window),
        }
    }

    #[allow(irrefutable_let_patterns)]
    pub fn build_histogram<T>(&self) -> Option<Histogram<T>>
    where
        T: Unsigned + SaturatingArithmetic + Default + FetchCompareStore,
        <T as Atomic>::Primitive: Default + PartialEq + Copy + From<u8>,
        u64: From<<T as Atomic>::Primitive>,
    {
        if let SummaryType::Histogram(max, precision, window) = self.inner {
            Some(Histogram::new(max, precision, window, None))
        } else {
            None
        }
    }
}
