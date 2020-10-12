use std::sync::Arc;
use std::time::Instant;
use rustcommon_atomichash::*;

pub fn main() {
	let loops = 10_000_000;
	let range = 1;
	let map = Arc::new(AtomicHashMap::<u64, u64>::with_capacity(range));
	let start = Instant::now();
	for _ in 0..loops {
		for i in 0..range {
			let _  = map.insert(i as u64, i as u64);
		}
	}
	let stop = Instant::now();
	let elapsed = stop - start;
	let elapsed = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
	let throughput = (loops * range) as f64 / elapsed;
	println!("rate: {} insert/s", throughput);
}