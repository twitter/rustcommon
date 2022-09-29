// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod error;
mod heatmap;
mod window;

use core::sync::atomic::AtomicU64;
use rustcommon_time::Nanoseconds;

pub use self::heatmap::Heatmap;
pub use error::Error;
pub use window::Window;

pub type Instant = rustcommon_time::Instant<Nanoseconds<u64>>;
pub type Duration = rustcommon_time::Duration<Nanoseconds<u64>>;

type AtomicInstant = rustcommon_time::Instant<Nanoseconds<AtomicU64>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_out() {
        let heatmap =
            Heatmap::new(0, 4, 20, Duration::from_secs(1), Duration::from_millis(1)).unwrap();
        assert_eq!(heatmap.percentile(0.0).map(|v| v.high()), Err(Error::Empty));
        heatmap.increment(Instant::now(), 1, 1);
        assert_eq!(heatmap.percentile(0.0).map(|v| v.high()), Ok(1));
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(heatmap.percentile(0.0).map(|v| v.high()), Ok(1));
        std::thread::sleep(std::time::Duration::from_millis(2000));
        assert_eq!(heatmap.percentile(0.0).map(|v| v.high()), Err(Error::Empty));
    }
}
