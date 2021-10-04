// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::lazy::Lazy;
use crate::Metric;
use core::cell::RefCell;
use os_thread_local::ThreadLocal;
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};

/// A counter which updates a thread local counter which can then be synced with
/// the global atomic counter. Can be incremented or added to. Unlike the normal
/// counter, this counter must by synced to the global atomic counter for
/// increments/adds to be visible to other threads. This can be done by calling
/// `sync()` or by loading the value.
///
/// In case of overflow the counter will wrap around. However, internally it
/// uses an unsigned 64-bit integer so for most use cases this should be
/// unlikely.
///
/// # Example
/// ```
/// # use rustcommon_metrics_v2::{metric, ThreadLocalCounter as Counter};
/// #[metric(name = "my.custom.metric")]
/// static MY_COUNTER: Counter = Counter::new();
///
/// fn a_method() {
///     MY_COUNTER.increment();
///     // ...
/// }
/// # a_method();
/// ```
pub struct Counter {
    global: AtomicU64,
    local: Lazy<ThreadLocal<RefCell<u64>>>,
}

impl Counter {
    /// Creates a new counter with thread local caching of writes.
    pub const fn new() -> Self {
        Self {
            global: AtomicU64::new(0),
            local: Lazy::new(|| ThreadLocal::new(|| RefCell::new(0))),
        }
    }

    /// Updates and reads the global counter. Requires one atomic operation.
    pub fn value(&self) -> u64 {
        let local = self.local.with(|v| *v.borrow());
        if local == 0 {
            self.global.load(Ordering::Relaxed)
        } else {
            let value = self.global.fetch_add(local, Ordering::Relaxed) + local;
            self.local.with(|v| *v.borrow_mut() = 0);
            value
        }
    }

    /// Synchronize the thread local counts with the global counter. Requires
    /// one atomic operation if not currently synced.
    pub fn sync(&self) {
        let local = self.local.with(|v| *v.borrow());
        if local != 0 {
            self.global.fetch_add(local, Ordering::Relaxed);
            self.local.with(|v| *v.borrow_mut() = 0);
        }
    }

    /// Adds a value to the counter. No atomic operations required.
    pub fn add(&self, value: u64) {
        self.local.with(|v| *v.borrow_mut() += value)
    }

    /// Increment the counter by one. No atomic operations required.
    pub fn increment(&self) {
        self.add(1)
    }
}

impl Metric for Counter {
    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}
