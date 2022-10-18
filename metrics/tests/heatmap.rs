// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_metrics::{heatmap, metrics};

heatmap!(LATENCY, 1_000_000_000);
heatmap!(CARDINALITY, 1_000, "some description");

#[test]
fn metric_name_as_expected() {
    let metrics = metrics().static_metrics();
    assert_eq!(metrics.len(), 2);
    assert_eq!(metrics[1].name(), "latency");
    assert_eq!(metrics[1].description(), None);
}

#[test]
fn metric_name_and_description_as_expected() {
    let metrics = metrics().static_metrics();
    assert_eq!(metrics.len(), 2);
    assert_eq!(metrics[0].name(), "cardinality");
    assert_eq!(metrics[0].description(), Some("some description"));
}
