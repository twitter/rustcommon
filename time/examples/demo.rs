// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_time::*;

pub fn main() {
    println!("precise: {:?}", Instant::<Nanoseconds<u64>>::recent());
    println!("coarse: {:?}", Instant::<Seconds<u32>>::recent());
    println!(
        "unix precise: {:?}",
        UnixInstant::<Nanoseconds<u64>>::recent()
    );
    println!("unix coarse: {:?}", UnixInstant::<Seconds<u32>>::recent());
    println!("utc: {}", DateTime::recent());
    std::thread::sleep(core::time::Duration::from_millis(50));
    refresh_clock();
    println!("precise: {:?}", Instant::<Nanoseconds<u64>>::recent());
    println!("coarse: {:?}", Instant::<Seconds<u32>>::recent());
    println!(
        "unix precise: {:?}",
        UnixInstant::<Nanoseconds<u64>>::recent()
    );
    println!("unix coarse: {:?}", UnixInstant::<Seconds<u32>>::recent());
    println!("utc: {}", DateTime::recent());
}
