// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_metrics::*;

#[metric(name = "custom-name")]
static METRIC: Counter = Counter::new();

#[metric(
    name = "custom-name-with-description",
    description = "some metric with a description"
)]
static METRIC_WITH_DESCRIPTION: Counter = Counter::new();

#[test]
fn metric_name_as_expected() {
    let metrics = metrics().static_metrics();
    assert_eq!(metrics.len(), 2);
    assert_eq!(metrics[1].name(), "custom-name");
    assert_eq!(metrics[1].description(), "");
}

#[test]
fn metric_name_and_description_as_expected() {
    let metrics = metrics().static_metrics();
    assert_eq!(metrics.len(), 2);
    assert_eq!(metrics[0].name(), "custom-name-with-description");
    assert_eq!(metrics[0].description(), "some metric with a description");
}
