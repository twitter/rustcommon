use criterion::BenchmarkId;
use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Criterion};
use rustcommon_histogram::*;

fn increment_u8(c: &mut Criterion) {
    let max = u8::MAX;

    let mut group = c.benchmark_group("AtomicHistogram/u8/AtomicU8/increment");

    for precision in 1..=3 {
        let histogram = AtomicHistogram::<u8, AtomicU8>::new(max, precision);
        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new("min/precision", precision), |b| {
            b.iter(|| histogram.increment(1, 1))
        });
        group.bench_function(BenchmarkId::new("max/precision", precision), |b| {
            b.iter(|| histogram.increment(max, 1))
        });
    }
}

fn increment_u16(c: &mut Criterion) {
    let max = u16::MAX;

    let mut group = c.benchmark_group("AtomicHistogram/u16/AtomicU16/increment");

    for precision in 1..=5 {
        let histogram = AtomicHistogram::<u16, AtomicU16>::new(max, precision);
        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new("min/precision", precision), |b| {
            b.iter(|| histogram.increment(1, 1))
        });
        group.bench_function(BenchmarkId::new("max/precision", precision), |b| {
            b.iter(|| histogram.increment(max, 1))
        });
    }
}

fn increment_u32(c: &mut Criterion) {
    let max = u32::MAX;

    let mut group = c.benchmark_group("AtomicHistogram/u32/AtomicU32/increment");

    for precision in 1..=7 {
        let histogram = AtomicHistogram::<u32, AtomicU32>::new(max, precision);
        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new("min/precision", precision), |b| {
            b.iter(|| histogram.increment(1, 1))
        });
        group.bench_function(BenchmarkId::new("max/precision", precision), |b| {
            b.iter(|| histogram.increment(max, 1))
        });
    }
}

fn increment_u64(c: &mut Criterion) {
    let max = u64::MAX;

    let mut group = c.benchmark_group("AtomicHistogram/u64/AtomicU64/increment");

    for precision in 1..=6 {
        let histogram = AtomicHistogram::<u64, AtomicU64>::new(max, precision);
        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new("min/precision", precision), |b| {
            b.iter(|| histogram.increment(1, 1))
        });
        group.bench_function(BenchmarkId::new("max/precision", precision), |b| {
            b.iter(|| histogram.increment(max, 1))
        });
    }
}

fn sub_assign_u8(c: &mut Criterion) {
    let max = u8::MAX;

    let mut group = c.benchmark_group("AtomicHistogram/u8/AtomicU8/sub_assign");

    for precision in 1..=3 {
        let alpha = AtomicHistogram::<u8, AtomicU8>::new(max, precision);
        let bravo = AtomicHistogram::<u8, AtomicU8>::new(max, precision);
        group.bench_function(BenchmarkId::new("fast/precision", precision), |b| {
            b.iter(|| alpha.sub_assign(&bravo))
        });
    }

    for precision in 1..3 {
        let alpha = AtomicHistogram::<u8, AtomicU8>::new(max, precision + 1);
        let bravo = AtomicHistogram::<u8, AtomicU8>::new(max, precision);
        group.bench_function(BenchmarkId::new("slow/precision", precision + 1), |b| {
            b.iter(|| alpha.sub_assign(&bravo))
        });
    }
}

fn sub_assign_u16(c: &mut Criterion) {
    let max = u16::MAX;

    let mut group = c.benchmark_group("AtomicHistogram/u16/AtomicU16/sub_assign");

    for precision in 1..=4 {
        let alpha = AtomicHistogram::<u16, AtomicU16>::new(max, precision);
        let bravo = AtomicHistogram::<u16, AtomicU16>::new(max, precision);
        group.bench_function(BenchmarkId::new("fast/precision", precision), |b| {
            b.iter(|| alpha.sub_assign(&bravo))
        });
    }

    for precision in 1..4 {
        let alpha = AtomicHistogram::<u16, AtomicU16>::new(max, precision + 1);
        let bravo = AtomicHistogram::<u16, AtomicU16>::new(max, precision);
        group.bench_function(BenchmarkId::new("slow/precision", precision + 1), |b| {
            b.iter(|| alpha.sub_assign(&bravo))
        });
    }
}

fn sub_assign_u32(c: &mut Criterion) {
    let max = u32::MAX;

    let mut group = c.benchmark_group("AtomicHistogram/u32/AtomicU32/sub_assign");

    for precision in 1..=7 {
        let alpha = AtomicHistogram::<u32, AtomicU32>::new(max, precision);
        let bravo = AtomicHistogram::<u32, AtomicU32>::new(max, precision);
        group.bench_function(BenchmarkId::new("fast/precision", precision), |b| {
            b.iter(|| alpha.sub_assign(&bravo))
        });
    }

    for precision in 1..7 {
        let alpha = AtomicHistogram::<u32, AtomicU32>::new(max, precision + 1);
        let bravo = AtomicHistogram::<u32, AtomicU32>::new(max, precision);
        group.bench_function(BenchmarkId::new("slow/precision", precision + 1), |b| {
            b.iter(|| alpha.sub_assign(&bravo))
        });
    }
}

fn sub_assign_u64(c: &mut Criterion) {
    let max = u64::MAX;

    let mut group = c.benchmark_group("AtomicHistogram/u64/AtomicU64/sub_assign");

    for precision in 1..=6 {
        let alpha = AtomicHistogram::<u64, AtomicU64>::new(max, precision);
        let bravo = AtomicHistogram::<u64, AtomicU64>::new(max, precision);
        group.bench_function(BenchmarkId::new("fast/precision", precision), |b| {
            b.iter(|| alpha.sub_assign(&bravo))
        });
    }

    for precision in 1..6 {
        let alpha = AtomicHistogram::<u64, AtomicU64>::new(max, precision + 1);
        let bravo = AtomicHistogram::<u64, AtomicU64>::new(max, precision);
        group.bench_function(BenchmarkId::new("slow/precision", precision + 1), |b| {
            b.iter(|| alpha.sub_assign(&bravo))
        });
    }
}

criterion_group!(
    benches,
    increment_u8,
    increment_u16,
    increment_u32,
    increment_u64,
    sub_assign_u8,
    sub_assign_u16,
    sub_assign_u32,
    sub_assign_u64,
);
criterion_main!(benches);
