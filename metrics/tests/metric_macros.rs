use rustcommon_metrics::{counter, gauge, heatmap};

counter!(A_COUNTER);
gauge!(A_GAUGE);
heatmap!(A_HEATMAP, 50);

#[test]
fn metrics_are_present() {
    let metrics = rustcommon_metrics::metrics();
    let metrics = metrics.static_metrics();

    assert_eq!(metrics.len(), 3);
    assert!(metrics.iter().any(|metric| metric.name() == "a_counter"));
    assert!(metrics.iter().any(|metric| metric.name() == "a_gauge"));
    assert!(metrics.iter().any(|metric| metric.name() == "a_heatmap"));
}
