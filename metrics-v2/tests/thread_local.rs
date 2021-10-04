// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_metrics_v2::*;
use rustcommon_metrics_v2::ThreadLocalCounter as Counter;

#[metric(name = "alpha")]
static ALPHA: Counter = Counter::new();

#[metric(name = "bravo")]
static BRAVO: Counter = Counter::new();

#[test]
fn thread_local_counter() {
    // basic behaviors when using only one thread
    assert_eq!(ALPHA.value(), 0);
    assert_eq!(BRAVO.value(), 0);
    ALPHA.increment();
    assert_eq!(ALPHA.value(), 1);
    assert_eq!(BRAVO.value(), 0);

    // if a thread exits without flushing, the count won't increment
    std::thread::spawn(move|| {
        ALPHA.increment();
        BRAVO.increment();
    }).join().unwrap();
    assert_eq!(ALPHA.value(), 1);
    assert_eq!(BRAVO.value(), 0);

    // as long as the thread flushes the counter, the new count is reflected
    std::thread::spawn(move|| {
        ALPHA.increment();
        BRAVO.increment();
        rustcommon_metrics_v2::sync();
    }).join().unwrap();
    assert_eq!(ALPHA.value(), 2);
    assert_eq!(BRAVO.value(), 1);

    // we can also operate on all the metrics to 
    std::thread::spawn(move|| {
        ALPHA.add(1);
        BRAVO.add(1);
        rustcommon_metrics_v2::sync();
    }).join().unwrap();
    assert_eq!(ALPHA.value(), 3);
    assert_eq!(BRAVO.value(), 2);
}

