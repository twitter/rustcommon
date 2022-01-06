// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod error;
mod heatmaps;
mod window;

pub use error::HeatmapError;
pub use heatmaps::{AtomicHeatmap, Heatmap};
pub use window::Window;

pub use rustcommon_atomics::{Atomic, AtomicU16, AtomicU32, AtomicU64, AtomicU8};
pub use rustcommon_histogram::{AtomicCounter, Counter, Indexing};
pub use rustcommon_time::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_out() {
        let mut heatmap =
            Heatmap::<u64, u64>::new(1_000_000, 2, Duration::<Nanoseconds<u64>>::from_secs(1), Duration::<Nanoseconds<u64>>::from_millis(1));
        assert_eq!(heatmap.percentile(0.0), Err(HeatmapError::Empty));
        heatmap.increment(Instant::<Nanoseconds<u64>>::now(), 1, 1);
        assert_eq!(heatmap.percentile(0.0), Ok(1));
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(heatmap.percentile(0.0), Ok(1));
        std::thread::sleep(std::time::Duration::from_millis(2000));
        assert_eq!(heatmap.percentile(0.0), Err(HeatmapError::Empty));

        let heatmap = AtomicHeatmap::<u64, AtomicU64>::new(
            1_000_000,
            2,
            Duration::<Nanoseconds<u64>>::from_secs(1),
            Duration::<Nanoseconds<u64>>::from_millis(1),
        );
        assert_eq!(heatmap.percentile(0.0), Err(HeatmapError::Empty));
        heatmap.increment(Instant::<Nanoseconds<u64>>::now(), 1, 1);
        assert_eq!(heatmap.percentile(0.0), Ok(1));
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(heatmap.percentile(0.0), Ok(1));
        std::thread::sleep(std::time::Duration::from_millis(2000));
        assert_eq!(heatmap.percentile(0.0), Err(HeatmapError::Empty));
    }
}
