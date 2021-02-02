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

impl rustcommon_fastmetrics::Metric for Metric {
    fn source(&self) -> Source {
        Source::Counter
    }

    fn index(&self) -> usize {
        (*self).into()
    }
}

impl Display for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Metric::Alpha => write!(f, "alpha"),
        }
    }
}

fn run(c: &mut Criterion) {
    MetricsBuilder::new().metric(Metric::Alpha).build().unwrap();

    let mut group = c.benchmark_group("Metrics/counter");

    group.throughput(Throughput::Elements(1));
    group.bench_function("set", |b| b.iter(|| set_counter!(&Metric::Alpha, 255)));
    group.bench_function("increment", |b| {
        b.iter(|| increment_counter!(&Metric::Alpha))
    });
}

criterion_group!(benches, run,);
criterion_main!(benches);
