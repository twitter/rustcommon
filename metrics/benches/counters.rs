use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Criterion};
use rustcommon_metrics::*;
use std::sync::Arc;
use std::time::Instant;

enum StatU8 {
    Alpha,
    Bravo,
}

impl Statistic<AtomicU8, AtomicU8> for StatU8 {
    fn name(&self) -> &str {
        match self {
            StatU8::Alpha => "alpha",
            StatU8::Bravo => "bravo",
        }
    }

    fn source(&self) -> Source {
        match self {
            StatU8::Alpha => Source::Counter,
            StatU8::Bravo => Source::Counter,
        }
    }

    fn summary(&self) -> Option<Summary<AtomicU8, AtomicU8>> {
        match self {
            StatU8::Bravo => Some(Summary::stream(1000)),
            _ => None,
        }
    }
}

fn u8_u8(c: &mut Criterion) {
    let metrics = Arc::new(Metrics::<AtomicU8, AtomicU8>::new());

    metrics.register(&StatU8::Alpha);
    metrics.add_output(&StatU8::Alpha, Output::Reading);

    metrics.register(&StatU8::Bravo);
    metrics.add_output(&StatU8::Bravo, Output::Reading);

    let mut group = c.benchmark_group("Metrics/AtomicU8/AtomicU8/counter");

    group.throughput(Throughput::Elements(1));
    let now = Instant::now();
    group.bench_function("no_summary/record", |b| {
        b.iter(|| metrics.record_counter(&StatU8::Alpha, now, 255))
    });
    group.bench_function("stream/1000/record", |b| {
        b.iter(|| metrics.record_counter(&StatU8::Bravo, now, 255))
    });
}

criterion_group!(benches, u8_u8,);
criterion_main!(benches);
