use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Criterion};
use rustcommon_heatmap::*;

type Instant = rustcommon_time::Instant<Nanoseconds<u64>>;
type Duration = rustcommon_time::Duration<Nanoseconds<u64>>;

fn u64_u64(c: &mut Criterion) {
    let mut heatmap =
        Heatmap::<u64, u64>::new(1_000_000, 2, Duration::from_secs(1), Duration::from_millis(1));

    let mut group = c.benchmark_group("Heatmap/u64/u64");

    group.throughput(Throughput::Elements(1));
    let mut time = Instant::now();
    let interval = Duration::from_millis(1);
    group.bench_function("increment", |b| {
        b.iter(|| {
            time += interval;
            heatmap.increment(Instant::now(), 1, 1)
        })
    });
}

fn atomic_u64_u64(c: &mut Criterion) {
    let heatmap = AtomicHeatmap::<u64, AtomicU64>::new(
        1_000_000,
        2,
        Duration::from_secs(1),
        Duration::from_millis(1),
    );

    let mut group = c.benchmark_group("AtomicHeatmap/u64/AtomicU64");

    group.throughput(Throughput::Elements(1));
    let mut time = Instant::now();
    let interval = Duration::from_millis(1);
    group.bench_function("increment", |b| {
        b.iter(|| {
            time += interval;
            heatmap.increment(Instant::now(), 1, 1)
        })
    });
}

criterion_group!(benches, u64_u64, atomic_u64_u64);
criterion_main!(benches);
