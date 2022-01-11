// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_ratelimiter::{Ratelimiter, Refill};
use rustcommon_time::{DateTime, SecondsFormat};

fn main() {
    for strategy in &[Refill::Normal, Refill::Uniform] {
        println!("strategy: {:?}", strategy);
        let limiter = Ratelimiter::new(1, 1, 1);
        limiter.set_strategy(*strategy);
        for i in 0..10 {
            limiter.wait();
            println!(
                "{}: T -{}",
                DateTime::now().to_rfc3339_opts(SecondsFormat::Millis, false),
                10 - i
            );
        }
        limiter.wait();
        println!();
    }
}
