
pub trait Value: Default + Copy + Ord { }

impl Value for u64 {}
impl Value for u32 {}
impl Value for u16 {}
impl Value for u8 {}
impl Value for usize {}

pub struct Streamstats<T> {
	buffer: Vec<T>,
	current: usize,
	oldest: usize,
}

impl<T> Streamstats<T>
where T: Value
{
	pub fn new(capacity: usize) -> Self {
		let mut buffer = Vec::with_capacity(capacity);
		for _ in 0..capacity {
			buffer.push(Default::default());
		}
		Self {
			buffer,
			current: 0,
			oldest: 0,
		}
	}

	pub fn insert(&mut self, value: T) {
		self.buffer[self.current] = value;
		self.current += 1;
		if self.current >= self.buffer.len() {
			self.current = 0;
		}
		if self.current == self.oldest {
			self.oldest += 1;
			if self.oldest >= self.buffer.len() {
				self.oldest = 0;
			}
		}
	}

	fn values(&self) -> usize {
		if self.current < self.oldest {
			(self.current + self.buffer.len()) - self.oldest
		} else if self.current == self.oldest {
			0
		} else {
			self.current - self.oldest
		}
	}

	pub fn percentile(&self, percentile: f64) -> Option<T> {
		let values = self.values();
		if values == 0 {
			None
		} else {
			let mut tmp = Vec::with_capacity(values);
			if self.current > self.oldest {
				for i in self.oldest..self.current {
					tmp.push(self.buffer[i]);
				}
			} else {
				for i in self.oldest..self.buffer.len() {
					tmp.push(self.buffer[i]);
				}
				for i in 0..self.current {
					tmp.push(self.buffer[i]);
				}
			}
			tmp.sort();
			if percentile == 0.0 {
				Some(tmp[0])
			} else {
				let need = (percentile * values as f64).ceil() as usize;
				Some(tmp[need - 1])
			}
		}
	}

	pub fn clear(&mut self) {
		self.oldest = self.current;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

    #[test]
    fn basic() {
        let mut streamstats = Streamstats::<u64>::new(1000);
        assert_eq!(streamstats.percentile(0.0), None);
        streamstats.insert(1);
        assert_eq!(streamstats.percentile(0.0), Some(1));
        streamstats.clear();
        for i in 0..=1_000_000 {
        	streamstats.insert(i);
        	assert_eq!(streamstats.percentile(1.0), Some(i));
        }
    }
}
