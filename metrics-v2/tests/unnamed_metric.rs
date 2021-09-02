// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_metrics_v2::*;

#[metric]
static TEST_METRIC: Counter = Counter::new();

#[test]
fn metric_name_as_expected() {
    let metrics = metrics();

    assert_eq!(metrics.len(), 1);
    assert_eq!(
        metrics[0].name(),
        concat!(module_path!(), "::", "TEST_METRIC")
    );
}
