// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

// Inspiration taken from Rust `log` crate: https://github.com/rust-lang/log

mod error;
#[macro_use]
mod macros;
mod metric;
mod metrics;
mod metrics_lib;
mod nop_metrics;

use crate::nop_metrics::NopMetrics;
use crate::metrics_lib::MetricsLib;

pub use error::MetricsError;
pub use macros::*;
pub use metrics::{Metrics, MetricsBuilder};
pub use metric::{Metric, Source};

use core::sync::atomic::AtomicUsize;
use core::sync::atomic::AtomicI64;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;

static mut METRICS: &dyn MetricsLib = &NopMetrics;
static STATE: AtomicUsize = AtomicUsize::new(UNINITIALIZED);

const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

pub fn metrics() -> &'static dyn MetricsLib {
    unsafe { METRICS }
}

#[cfg(test)]
mod tests {
    use core::fmt::Display;
    use super::*;

    #[derive(Copy, Clone)]
    #[allow(dead_code)]
    enum Metric {
        Alpha,
        Bravo,
        Charlie,
    }

    impl Into<usize> for Metric {
        fn into(self) -> usize {
            self as usize
        }
    }

    impl Display for Metric {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                Metric::Alpha => write!(f, "alpha"),
                Metric::Bravo => write!(f, "bravo"),
                Metric::Charlie => write!(f, "charlie"),
            }
        }
    }

    impl super::Metric for Metric {
        fn source(&self) -> Source {
            Source::Counter
        }

        fn index(&self) -> usize {
            (*self).into()
        }
    }

    // #[test]
    // fn counter() {
    //     let metrics = MetricsBuilder::new()
    //         .metrics(&[Metric::Alpha, Metric::Charlie])
    //         .build();

    //     assert_eq!(metrics.get_counter(&Metric::Alpha), Ok(0));
    //     metrics.set_counter(&Metric::Alpha, 100);
    //     assert_eq!(metrics.get_counter(&Metric::Alpha), Ok(100));

    //     assert_eq!(metrics.get_counter(&Metric::Charlie), Ok(0));
    //     metrics.increment_counter(&Metric::Charlie, 1337);
    //     assert_eq!(metrics.get_counter(&Metric::Charlie), Ok(1337));
    // }

    #[test]
    fn macros() {
        MetricsBuilder::new()
            .metrics(&[Metric::Alpha, Metric::Charlie])
            .build().unwrap();
        set_counter!(&Metric::Alpha, 100);
        assert_eq!(get_counter!(&Metric::Alpha), Ok(100));
        increment_counter!(&Metric::Alpha);
        assert_eq!(get_counter!(&Metric::Alpha), Ok(101));
        increment_counter_by!(&Metric::Alpha, 99);
        assert_eq!(get_counter!(&Metric::Alpha), Ok(200));
    }
}
