// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Criterion};
use rustcommon_time::*;

fn instant_seconds_u32(c: &mut Criterion) {
    let mut group = c.benchmark_group("Instant<Seconds<u32>>");

    group.throughput(Throughput::Elements(1));
    group.bench_function("now", |b| b.iter(UnixInstant::<Seconds<u32>>::now));
    group.bench_function("recent", |b| b.iter(UnixInstant::<Seconds<u32>>::recent));
}

fn instant_nanoseconds_u64(c: &mut Criterion) {
    let mut group = c.benchmark_group("Instant<Nanoseconds<u64>>");

    group.throughput(Throughput::Elements(1));
    group.bench_function("now", |b| b.iter(UnixInstant::<Nanoseconds<u64>>::now));
    group.bench_function("recent", |b| {
        b.iter(UnixInstant::<Nanoseconds<u64>>::recent)
    });
}

fn datetime(c: &mut Criterion) {
    let mut group = c.benchmark_group("DateTime");

    group.throughput(Throughput::Elements(1));
    group.bench_function("now", |b| b.iter(DateTime::now));
    group.bench_function("recent", |b| b.iter(DateTime::recent));
}

fn refresh(c: &mut Criterion) {
    let mut group = c.benchmark_group("Clock");

    group.throughput(Throughput::Elements(1));
    group.bench_function("refresh", |b| b.iter(refresh_clock));
}

criterion_group!(
    benches,
    instant_seconds_u32,
    instant_nanoseconds_u64,
    datetime,
    refresh
);
criterion_main!(benches);
