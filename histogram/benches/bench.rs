// Copyright 2022 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use histogram::Histogram;

fn increment(c: &mut Criterion) {
    let histogram = Histogram::new(0, 10, 30).unwrap();

    let mut group = c.benchmark_group("histogram/increment");
    group.throughput(Throughput::Elements(1));

    group.throughput(Throughput::Elements(1));
    group.bench_function("min", |b| b.iter(|| histogram.increment(1, 1)));
    group.bench_function("max", |b| b.iter(|| histogram.increment(1073741823, 1)));
}

criterion_group!(benches, increment,);
criterion_main!(benches);
