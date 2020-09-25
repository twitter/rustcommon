use criterion::{criterion_group, criterion_main, Criterion};
use rustcommon_histogram::Histogram;

fn increment_u8(c: &mut Criterion) {
    let max = 100;

    let mut histogram = Histogram::<u8, u8>::new(max, 1);
    c.bench_function("Histogram::<u8, u8>::increment() precision 1 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u8, u8>::increment() precision 1 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });

    let mut histogram = Histogram::<u8, u8>::new(max, 2);
    c.bench_function("Histogram::<u8, u8>::increment() precision 2 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u8, u8>::increment() precision 2 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });

    let mut histogram = Histogram::<u8, u8>::new(max, 3);
    c.bench_function("Histogram::<u8, u8>::increment() precision 3 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u8, u8>::increment() precision 3 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });
}

fn increment_u16(c: &mut Criterion) {
    let max = 10_000;

    let mut histogram = Histogram::<u16, u16>::new(max, 1);
    c.bench_function("Histogram::<u16, u16>::increment() precision 1 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u16, u16>::increment() precision 1 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });

    let mut histogram = Histogram::<u16, u16>::new(max, 2);
    c.bench_function("Histogram::<u16, u16>::increment() precision 2 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u16, u16>::increment() precision 2 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });

    let mut histogram = Histogram::<u16, u16>::new(max, 3);
    c.bench_function("Histogram::<u16, u16>::increment() precision 3 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u16, u16>::increment() precision 3 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });

    let mut histogram = Histogram::<u16, u16>::new(max, 4);
    c.bench_function("Histogram::<u16, u16>::increment() precision 4 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u16, u16>::increment() precision 4 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });

    let mut histogram = Histogram::<u16, u16>::new(max, 5);
    c.bench_function("Histogram::<u16, u16>::increment() precision 5 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u16, u16>::increment() precision 5 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });
}

fn increment_u32(c: &mut Criterion) {
    let max = 1_000_000_000;

    let mut histogram = Histogram::<u32, u32>::new(max, 1);
    c.bench_function("Histogram::<u32, u32>::increment() precision 1 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u32, u32>::increment() precision 1 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });

    let mut histogram = Histogram::<u32, u32>::new(max, 2);
    c.bench_function("Histogram::<u32, u32>::increment() precision 2 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u32, u32>::increment() precision 2 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });

    let mut histogram = Histogram::<u32, u32>::new(max, 3);
    c.bench_function("Histogram::<u32, u32>::increment() precision 3 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u32, u32>::increment() precision 3 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });

    let mut histogram = Histogram::<u32, u32>::new(max, 4);
    c.bench_function("Histogram::<u32, u32>::increment() precision 4 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u32, u32>::increment() precision 4 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });

    let mut histogram = Histogram::<u32, u32>::new(max, 5);
    c.bench_function("Histogram::<u32, u32>::increment() precision 5 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u32, u32>::increment() precision 5 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });

    let mut histogram = Histogram::<u32, u32>::new(max, 6);
    c.bench_function("Histogram::<u32, u32>::increment() precision 6 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u32, u32>::increment() precision 6 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });
}

fn increment_u64(c: &mut Criterion) {
    let max = 1_000_000_000;

    let mut histogram = Histogram::<u64, u64>::new(max, 1);
    c.bench_function("Histogram::<u64, u64>::increment() precision 1 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u64, u64>::increment() precision 1 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });

    let mut histogram = Histogram::<u64, u64>::new(max, 2);
    c.bench_function("Histogram::<u64, u64>::increment() precision 2 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u64, u64>::increment() precision 2 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });

    let mut histogram = Histogram::<u64, u64>::new(max, 3);
    c.bench_function("Histogram::<u64, u64>::increment() precision 3 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u64, u64>::increment() precision 3 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });

    let mut histogram = Histogram::<u64, u64>::new(max, 4);
    c.bench_function("Histogram::<u64, u64>::increment() precision 4 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u64, u64>::increment() precision 4 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });

    let mut histogram = Histogram::<u64, u64>::new(max, 5);
    c.bench_function("Histogram::<u64, u64>::increment() precision 5 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u64, u64>::increment() precision 5 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });

    let mut histogram = Histogram::<u64, u64>::new(max, 6);
    c.bench_function("Histogram::<u64, u64>::increment() precision 6 min", |b| {
        b.iter(|| histogram.increment(1, 1))
    });
    c.bench_function("Histogram::<u64, u64>::increment() precision 6 max", |b| {
        b.iter(|| histogram.increment(max, 1))
    });
}

fn subtract(c: &mut Criterion) {
    let max = 1_000_000_000;

    let mut alpha = Histogram::<u64, u64>::new(max, 6);
    let bravo = alpha.clone();

    c.bench_function("Histogram::<u64, u64>::increment() precision 6 min", |b| {
        b.iter(|| alpha.sub_assign(&bravo))
    });
}

criterion_group!(
    benches,
    increment_u8,
    increment_u16,
    increment_u32,
    increment_u64,
    subtract,
);
criterion_main!(benches);
