// Copyright 2022 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_metrics::*;

#[metric(name = "pid")]
static PID: Gauge = Gauge::new();

#[metric(name = "composed/response", namespace = "server")]
static COMPOSED_RESPONSE: Counter = Counter::new();

#[test]
fn without_namespace() {
    let metrics = metrics().static_metrics();
    assert_eq!(metrics.len(), 2);
    assert_eq!(metrics[1].name(), "pid");
    assert_eq!(metrics[1].namespace(), None);
}

#[test]
fn with_namespace() {
    let metrics = metrics().static_metrics();
    assert_eq!(metrics.len(), 2);
    assert_eq!(metrics[0].name(), "composed/response");
    assert_eq!(metrics[0].namespace(), Some("server"));
}
