// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Metric;
use once_cell::sync::OnceCell;
use std::cell::Cell;
use std::ops::{Deref, DerefMut};

// Note: This implementation is mostly copied from the Lazy implementation
//       within once_cell. It only adds the `get` option to try to access
//       the value without initializing the Lazy instance.
//
//       This should be replaced with the new primitives in std::lazy once
//       those stabilize.

/// A value which is initialized on the first access.
///
/// This type is thread-safe and can be used in statics.
/// 
/// # Example
/// In this example, [`Heatmap`] does not have a const `new` function so it
/// must be constructed using [`Lazy`].
/// ```
/// # #[cfg(feature = "heatmap")]
/// # fn main() {
/// # use rustcommon_metrics_v2::*;
/// # use std::time::Duration;
/// #[metric]
/// static HEATMAP: Lazy<Heatmap> = Lazy::new(|| Heatmap::new(
///     100, 2, Duration::from_secs(30), Duration::from_secs(1)
/// ));
/// # }
/// # #[cfg(not(feature = "heatmap"))] fn main() {}
/// ```
/// 
/// [`Heatmap`]: crate::Heatmap;
pub struct Lazy<T, F = fn() -> T> {
    cell: OnceCell<T>,
    func: Cell<Option<F>>,
}

unsafe impl<T, F: Send> Sync for Lazy<T, F> where OnceCell<T>: Sync {}

impl<T, F> Lazy<T, F> {
    /// Create a new lazy value with the given initializing function.
    pub const fn new(func: F) -> Self {
        Self {
            cell: OnceCell::new(),
            func: Cell::new(Some(func)),
        }
    }

    /// If this lazy has been initialized, then return a reference to the
    /// contained value.
    pub fn get(this: &Self) -> Option<&T> {
        this.cell.get()
    }
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    /// Force the evaluation of this lazy value and return a reference to
    /// the result. This is equivalent to the `Deref` impl.
    pub fn force(this: &Self) -> &T {
        this.cell.get_or_init(|| {
            let func = this
                .func
                .take()
                .unwrap_or_else(|| panic!("Lazy instance has previously been poisoned"));

            func()
        })
    }
}

impl<T, F: FnOnce() -> T> Deref for Lazy<T, F> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Lazy::force(self)
    }
}

impl<T, F: FnOnce() -> T> DerefMut for Lazy<T, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Lazy::force(self);
        self.cell.get_mut().unwrap_or_else(|| unreachable!())
    }
}

impl<T: Default> Default for Lazy<T> {
    /// Create a new lazy value using `default` as the initializing function.
    fn default() -> Lazy<T> {
        Lazy::new(T::default)
    }
}

impl<T: Metric, F: Send + 'static> Metric for Lazy<T, F> {
    fn is_enabled(&self) -> bool {
        Lazy::get(self).is_some()
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        match Lazy::get(self) {
            Some(metric) => Some(metric),
            None => None,
        }
    }
}
