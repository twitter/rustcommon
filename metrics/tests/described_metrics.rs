// Copyright 2022 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_metrics::*;

#[metric(description = "some metric with a description")]
static METRIC_WITH_DESCRIPTION_NO_NAME: Counter = Counter::new();

#[test]
fn metric_description_as_expected_when_only_description_set() {
    let metrics = metrics().static_metrics();
    assert_eq!(metrics.len(), 1);
    assert_eq!(metrics[0].description(), "some metric with a description");
}
