// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_time::*;

type PreciseInstant = Instant<Nanoseconds<u64>>;
type CoarseInstant = Instant<Seconds<u32>>;
type PreciseUnix = UnixInstant<Nanoseconds<u64>>;
type CoarseUnix = UnixInstant<Seconds<u32>>;

pub fn main() {
    println!("precise: {:?}", PreciseInstant::recent());
    println!("coarse: {:?}", CoarseInstant::recent());
    println!("unix precise: {:?}", PreciseUnix::recent());
    println!("unix coarse: {:?}", CoarseUnix::recent());
    println!("utc: {}", DateTime::recent());
    std::thread::sleep(core::time::Duration::from_millis(50));
    refresh_clock();
    println!("precise: {:?}", PreciseInstant::recent());
    println!("coarse: {:?}", CoarseInstant::recent());
    println!("unix precise: {:?}", PreciseUnix::recent());
    println!("unix coarse: {:?}", CoarseUnix::recent());
    println!("utc: {}", DateTime::recent());
}
