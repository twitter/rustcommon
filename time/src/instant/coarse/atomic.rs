// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

use core::sync::atomic::{AtomicU32, Ordering};

/// `AtomicCoarseInstant` is an opaque type that represents a moment in time to
/// the nearest second. Unlike `CoarseInstant`, it is thread-safe.
#[derive(Debug)]
pub struct AtomicCoarseInstant {
    pub(crate) secs: AtomicU32, // This is enough for >100 years without overflow
}

impl AtomicCoarseInstant {
    pub fn now() -> Self {
        let instant = CoarseInstant::now();
        Self {
            secs: AtomicU32::new(instant.secs),
        }
    }

    pub fn recent() -> Self {
        let instant = CLOCK.recent_coarse();
        Self {
            secs: AtomicU32::new(instant.secs),
        }
    }

    pub fn load(&self, ordering: Ordering) -> CoarseInstant {
        CoarseInstant {
            secs: self.secs.load(ordering),
        }
    }

    pub fn store(&self, value: CoarseInstant, ordering: Ordering) {
        self.secs.store(value.secs, ordering)
    }

    pub fn fetch_add(&self, other: CoarseDuration, ordering: Ordering) -> CoarseInstant {
        CoarseInstant {
            secs: self.secs.fetch_add(other.secs, ordering),
        }
    }

    pub fn fetch_sub(&self, other: CoarseDuration, ordering: Ordering) -> CoarseInstant {
        CoarseInstant {
            secs: self.secs.fetch_sub(other.secs, ordering),
        }
    }

    pub fn refresh(&self, ordering: Ordering) {
        self.store(CoarseInstant::now(), ordering)
    }

    pub fn elapsed(&self, ordering: Ordering) -> CoarseDuration {
        self.load(ordering).elapsed()
    }

    pub fn swap(&self, value: CoarseInstant, ordering: Ordering) -> CoarseInstant {
        CoarseInstant {
            secs: self.secs.swap(value.secs, ordering),
        }
    }
}
