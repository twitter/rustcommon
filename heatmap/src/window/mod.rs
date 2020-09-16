use rustcommon_histogram::Histogram;
use std::time::Instant;

pub struct Window<Value, Count> {
    pub(crate) start: Instant,
    pub(crate) stop: Instant,
    pub(crate) histogram: Histogram<Value, Count>,
}

impl<Value, Count> Window<Value, Count> {
    pub fn start(&self) -> Instant {
        self.start
    }

    pub fn stop(&self) -> Instant {
        self.stop
    }

    pub fn histogram(&self) -> &Histogram<Value, Count> {
        &self.histogram
    }
}
