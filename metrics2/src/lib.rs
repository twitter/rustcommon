// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod channel;
mod metrics;
mod source;
mod summary;
mod traits;

use channel::Channel;
pub use metrics::Metrics;
pub use source::Source;
pub use summary::Summary;
pub use traits::*;

pub use rustcommon_atomics::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    enum TestStat {
        Alpha,
    }

    impl Statistic<AtomicU64, AtomicU64> for TestStat {
        fn name(&self) -> &str {
            match self {
                Self::Alpha => "alpha",
            }
        }

        fn source(&self) -> Source {
            match self {
                Self::Alpha => Source::Counter,
            }
        }

        fn summary(&self) -> Option<Summary<AtomicU64, AtomicU64>> {
            match self {
                Self::Alpha => Some(Summary::stream(1000)),
            }
        }
    }

    #[test]
    fn basic() {
        let metrics = Metrics::<AtomicU64, AtomicU64>::new();
        metrics.register(&TestStat::Alpha);
        assert!(metrics.reading(&TestStat::Alpha).is_err());
        metrics
            .record_counter(&TestStat::Alpha, Instant::now(), 0)
            .expect("failed to record counter");
        assert_eq!(metrics.reading(&TestStat::Alpha), Ok(0));
        let now = Instant::now();
        metrics
            .record_counter(&TestStat::Alpha, now, 0)
            .expect("failed to record counter");
        assert_eq!(metrics.reading(&TestStat::Alpha), Ok(0));
        assert_eq!(metrics.percentile(&TestStat::Alpha, 0.0), Ok(0));
        metrics
            .record_counter(&TestStat::Alpha, now + Duration::from_millis(1000), 1)
            .expect("failed to record counter");
        assert_eq!(metrics.reading(&TestStat::Alpha), Ok(1));
        assert_eq!(metrics.percentile(&TestStat::Alpha, 100.0), Ok(1));
    }
}
