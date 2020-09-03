pub trait Value: Default + Copy + Ord {}

impl Value for u64 {}
impl Value for u32 {}
impl Value for u16 {}
impl Value for u8 {}
impl Value for usize {}

pub struct Streamstats<T> {
    buffer: Vec<T>,
    current: usize,
    oldest: usize,
    sorted: Vec<T>,
}

impl<T> Streamstats<T>
where
    T: Value,
{
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        let sorted = buffer.clone();
        for _ in 0..capacity {
            buffer.push(Default::default());
        }
        Self {
            buffer,
            current: 0,
            oldest: 0,
            sorted,
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
        self.sorted.clear(); // resort required
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

    pub fn percentile(&mut self, percentile: f64) -> Option<T> {
        if self.sorted.len() == 0 {
            let values = self.values();
            if values == 0 {
                return None;
            } else {
                if self.current > self.oldest {
                    for i in self.oldest..self.current {
                        self.sorted.push(self.buffer[i]);
                    }
                } else {
                    for i in self.oldest..self.buffer.len() {
                        self.sorted.push(self.buffer[i]);
                    }
                    for i in 0..self.current {
                        self.sorted.push(self.buffer[i]);
                    }
                }
                self.sorted.sort();
            }
        }
        if percentile == 0.0 {
            Some(self.sorted[0])
        } else {
            let need = (percentile * self.sorted.len() as f64).ceil() as usize;
            Some(self.sorted[need - 1])
        }
    }

    pub fn clear(&mut self) {
        self.oldest = self.current;
        self.sorted.clear();
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
        assert_eq!(streamstats.percentile(0.0), None);

        for i in 0..=10_000 {
            streamstats.insert(i);
            assert_eq!(streamstats.percentile(1.0), Some(i));
        }
    }
}
