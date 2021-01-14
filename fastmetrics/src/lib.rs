// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::fmt::Display;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::AtomicI64;
use core::sync::atomic::Ordering;

pub trait Metric: Copy + Into<usize> + Display {}

pub enum Source {
	Counter,
	Gauge,
}

pub struct Metrics<T>
where T: Metric,
{
	channels: Vec<Option<Channel<T>>>,
}

pub enum Channel<T>
where T: Metric
{
	Counter { counter: AtomicU64, metric: T },
	Gauge { gauge: AtomicI64, metric: T },
}

#[derive(Default)]
pub struct MetricsBuilder<T>
where T: Metric
{
	channels: Vec<Option<Channel<T>>>,
}

impl<T> MetricsBuilder<T>
where T: Metric
{
	pub fn new() -> Self {
		Self {
			channels: Vec::new()
		}
	}

	pub fn counter(mut self, metric: T) -> Self {
		let id: usize = metric.into();
		for _ in self.channels.len()..=id {
			self.channels.push(None);
		}

		self.channels[id] = Some(Channel::Counter { counter: Default::default(), metric });

		self
	}

	pub fn gauge(mut self, metric: T) -> Self {
		let id: usize = metric.into();
		for _ in self.channels.len()..=id {
			self.channels.push(None);
		}

		self.channels[id] = Some(Channel::Gauge { gauge: Default::default(), metric });

		self
	}

	pub fn build(self) -> Metrics<T> {
		Metrics {
			channels: self.channels
		}
	}
}

impl<T> Metrics<T>
where T: Metric,
{

	fn get(&self, metric: T) -> Option<&Channel<T>> {
		match self.channels.get(metric.into()) {
			None | Some(None) => None,
			Some(c) => c.as_ref(),
		}
	}

	pub fn get_counter(&self, metric: T) -> u64 {
		match self.get(metric) {
			None => {
				panic!("metric ({}) is not registered", metric);
			}
			Some(Channel::Counter { counter, .. }) => {
				counter.load(Ordering::Relaxed)
			}
			_ => {
				panic!("metric ({}) is not a counter", metric);
			}
		}
	}

	pub fn record_counter(&self, metric: T, value: u64) {
		match self.get(metric) {
			None => {
				panic!("metric ({}) is not registered", metric);
			}
			Some(Channel::Counter { counter, .. }) => {
				counter.store(value, Ordering::Relaxed);
			}
			_ => {
				panic!("metric ({}) is not a counter", metric);
			}
		}
	}

	pub fn increment_counter(&self, metric: T, value: u64) {
		match self.get(metric) {
			None => {
				panic!("metric ({}) is not registered", metric);
			}
			Some(Channel::Counter { counter, .. }) => {
				counter.fetch_add(value, Ordering::Relaxed);
			}
			_ => {
				panic!("metric ({}) is not a counter", metric);
			}
		}
	}

	pub fn get_gauge(&self, metric: T) -> i64 {
		match self.get(metric) {
			None => {
				panic!("metric ({}) is not registered", metric);
			}
			Some(Channel::Gauge { gauge, .. }) => {
				gauge.load(Ordering::Relaxed)
			}
			_ => {
				panic!("metric ({}) is not a gauge", metric);
			}
		}
	}

	pub fn record_gauge(&self, metric: T, value: i64) {
		match self.get(metric) {
			None => {
				panic!("metric ({}) is not registered", metric);
			}
			Some(Channel::Gauge { gauge, .. }) => {
				gauge.store(value, Ordering::Relaxed);
			}
			_ => {
				panic!("metric ({}) is not a gauge", metric);
			}
		}
	}

	pub fn increment_gauge(&self, metric: T, value: i64) {
		match self.get(metric) {
			None => {
				panic!("metric ({}) is not registered", metric);
			}
			Some(Channel::Gauge { gauge, .. }) => {
				gauge.fetch_add(value, Ordering::Relaxed);
			}
			_ => {
				panic!("metric ({}) is not a gauge", metric);
			}
		}
	}

	pub fn decrement_gauge(&self, metric: T, value: i64) {
		match self.get(metric) {
			None => {
				panic!("metric ({}) is not registered", metric);
			}
			Some(Channel::Gauge { gauge, .. }) => {
				gauge.fetch_sub(value, Ordering::Relaxed);
			}
			_ => {
				panic!("metric ({}) is not a gauge", metric);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Copy, Clone)]
	#[allow(dead_code)]
	enum Metric {
		Alpha,
		Bravo,
		Charlie,
	}

	impl Into<usize> for Metric {
	    fn into(self) -> usize {
	        self as usize
	    }
	}

	impl Display for Metric {
	    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	        match self {
	            Metric::Alpha => write!(f, "alpha"),
	            Metric::Bravo => write!(f, "bravo"),
	            Metric::Charlie => write!(f, "charlie"),
	        }
	    }
	}

	impl super::Metric for Metric {}

    #[test]
    fn counter() {
        let metrics = MetricsBuilder::new().counter(Metric::Alpha).counter(Metric::Charlie).build();

        assert_eq!(metrics.get_counter(Metric::Alpha), 0);
        metrics.record_counter(Metric::Alpha, 100);
        assert_eq!(metrics.get_counter(Metric::Alpha), 100);

        assert_eq!(metrics.get_counter(Metric::Charlie), 0);
        metrics.increment_counter(Metric::Charlie, 1337);
        assert_eq!(metrics.get_counter(Metric::Charlie), 1337);
    }
}
