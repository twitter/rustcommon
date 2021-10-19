// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_time::{now_utc, SecondsFormat};
use rustcommon_ratelimiter::Ratelimiter;

fn main() {
    let limiter = Ratelimiter::new(1, 1, 1);
    for i in 0..10 {
        limiter.wait();
        println!("{}: T -{}", now_utc().to_rfc3339_opts(SecondsFormat::Millis, false), 10 - i);
    }
    limiter.wait();
    println!("Ignition");
}
