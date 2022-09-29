use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Criterion};
use heatmap::*;

fn heatmap(c: &mut Criterion) {
    let heatmap = Heatmap::new(0, 4, 20, Duration::from_secs(1), Duration::from_millis(1));

    let mut group = c.benchmark_group("heatmap");

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

criterion_group!(benches, heatmap);
criterion_main!(benches);
