use crate::Metric;
use rustcommon_histogram::AtomicHistogram;
use rustcommon_atomics::AtomicU64;


pub type Histogram = AtomicHistogram<u64, AtomicU64>;

impl<V, C> Metric for AtomicHistogram<V, C>
where
    V: Send + Sync + 'static,
    C: Send + Sync + 'static,
{
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}
