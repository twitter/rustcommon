use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Criterion};
use rustcommon_histogram2::Histogram;

fn increment(c: &mut Criterion) {
    let histogram = Histogram::new(0, 10, 30);

    let mut group = c.benchmark_group("histogram/increment");
    group.throughput(Throughput::Elements(1));

    group.throughput(Throughput::Elements(1));
    group.bench_function("min", |b| b.iter(|| histogram.increment(1, 1)));
    group.bench_function("max", |b| b.iter(|| histogram.increment(1073741823, 1)));
}

criterion_group!(benches, increment,);
criterion_main!(benches);
