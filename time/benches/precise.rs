// use criterion::BenchmarkId;
use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Criterion};
use rustcommon_time::*;

fn plain(c: &mut Criterion) {
    let mut group = c.benchmark_group("Instant");

    group.throughput(Throughput::Elements(1));
    group.bench_function("now", |b| {
        b.iter(Instant::now)
    });
    group.bench_function("recent", |b| {
        b.iter(Instant::recent)
    });
}

fn atomic(c: &mut Criterion) {
    let mut group = c.benchmark_group("AtomicInstant");

    group.throughput(Throughput::Elements(1));
    group.bench_function("now", |b| {
        b.iter(AtomicInstant::now)
    });
    group.bench_function("recent", |b| {
        b.iter(AtomicInstant::recent)
    });
}

criterion_group!(
    benches,
    plain,
    atomic,
);
criterion_main!(benches);
