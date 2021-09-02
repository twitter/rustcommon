use rustcommon_metrics_v2::*;

#[metric(name = "custom-name")]
static METRIC: Counter = Counter::new();

#[test]
fn metric_name_as_expected() {
    let metrics = metrics();
    assert_eq!(metrics.len(), 1);
    assert_eq!(metrics[0].name(), "custom-name");
}
