// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::fmt::Display;
use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Criterion};
use rustcommon_fastmetrics::*;

#[derive(Copy, Clone)]
enum Metric {
    Alpha,
}

impl Into<usize> for Metric {
    fn into(self) -> usize {
        self as usize
    }
}

impl rustcommon_fastmetrics::Metric for Metric {}

impl Display for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Metric::Alpha => write!(f, "alpha"),
        }
    }
}

fn run(c: &mut Criterion) {
    let metrics = MetricsBuilder::new().counter(Metric::Alpha).build();

    let mut group = c.benchmark_group("Metrics/counter");

    group.throughput(Throughput::Elements(1));
    group.bench_function("record", |b| {
        b.iter(|| metrics.record_counter(Metric::Alpha, 255))
    });
}

criterion_group!(benches, run,);
criterion_main!(benches);
