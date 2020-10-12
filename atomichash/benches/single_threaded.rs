use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::thread_rng;
use rustcommon_atomichash::AtomicHashMap;
use std::sync::Arc;

fn u64_u64(c: &mut Criterion) {
    let map = Arc::new(AtomicHashMap::<u64, u64>::with_capacity(1000000));

    let mut group = c.benchmark_group("atomichash/u64/u64");

    group.throughput(Throughput::Elements(1));

    let mut rng = thread_rng();
    let distribution = Uniform::new_inclusive(0.0, u32::MAX as f64);

    let mut value = distribution.sample(&mut rng).floor() as u64;
    // pre-fill to 50% occupancy
    for _ in 0..500000 {
        let _ = map.insert(value, value);
        value = distribution.sample(&mut rng).floor() as u64;
    }

    group.bench_function("insert/50%", |b| {
        b.iter(|| {
            let _ = map.insert(value, value);
        })
    });

    group.bench_function("get/50%", |b| {
        b.iter(|| {
            map.get(&value);
        })
    });
}

fn std_hashmap(c: &mut Criterion) {
    let mut map = std::collections::HashMap::new();

    let mut group = c.benchmark_group("std/u64/u64");

    group.throughput(Throughput::Elements(1));

    let mut rng = thread_rng();
    let distribution = Uniform::new_inclusive(0.0, u32::MAX as f64);

    let mut value = distribution.sample(&mut rng).floor() as u64;
    // pre-fill to 50% occupancy
    for _ in 0..500000 {
        map.insert(value, value);
        value = distribution.sample(&mut rng).floor() as u64;
    }

    group.bench_function("insert/50%", |b| {
        b.iter(|| {
            map.insert(value, value);
        })
    });

    group.bench_function("get/50%", |b| {
        b.iter(|| {
            map.get(&value);
        })
    });
}

fn dashmap(c: &mut Criterion) {
    let map = dashmap::DashMap::new();

    let mut group = c.benchmark_group("dashmap/u64/u64");

    group.throughput(Throughput::Elements(1));

    let mut rng = thread_rng();
    let distribution = Uniform::new_inclusive(0.0, u32::MAX as f64);

    let mut value = distribution.sample(&mut rng).floor() as u64;
    // pre-fill to 50% occupancy
    for _ in 0..500000 {
        map.insert(value, value);
        value = distribution.sample(&mut rng).floor() as u64;
    }

    group.bench_function("insert/50%", |b| {
        b.iter(|| {
            map.insert(value, value);
        })
    });

    group.bench_function("get/50%", |b| {
        b.iter(|| {
            map.get(&value);
        })
    });
}

criterion_group!(benches, u64_u64, std_hashmap, dashmap);
criterion_main!(benches);
