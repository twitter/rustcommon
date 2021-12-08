// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_time::*;

pub fn main() {
    println!("precise: {:?}", recent_precise());
    println!("coarse: {:?}", recent_coarse());
    println!("system: {:?}", recent_system());
    println!("unix coarse: {:?}", recent_unix());
    println!("unix precise: {:?}", recent_unix_precise());
    println!("utc: {}", recent_utc());
    std::thread::sleep(core::time::Duration::from_millis(50));
    refresh_clock();
    println!("precise: {:?}", recent_precise());
    println!("coarse: {:?}", recent_coarse());
    println!("system: {:?}", recent_system());
    println!("unix coarse: {:?}", recent_unix());
    println!("unix precise: {:?}", recent_unix_precise());
    println!("utc: {}", recent_utc());
}
