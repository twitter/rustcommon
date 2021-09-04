mod bonus {
    pub use rustcommon_metrics_v2::*;
}

use bonus::Counter;

#[bonus::metric(name = "test", crate = crate::bonus)]
static METRIC: Counter = Counter::new();

macro_rules! metric_in_macro {
    () => {
        #[$crate::bonus::metric(crate = $crate::bonus)]
        static OTHER_METRIC: Counter = Counter::new();
    };
}

metric_in_macro!();
