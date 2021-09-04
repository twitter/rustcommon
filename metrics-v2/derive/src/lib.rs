// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use proc_macro::TokenStream;

mod metric;

/// Declare a global metric that can be accessed via the `metrics` method.
///
/// # Parameters
/// - (optional) `name`: The string name that the metric should be exposed as.
///   If not specified then the default name is one based on the path to the
///   metric along with its name.
/// - (optional) `crate`: The path to the `rustcommon_metrics_v2` crate. This
///   allows the `metric` macro to be used within other macros that get exported
///   to third-party crates which may not have added `rustcommon_metrics_v2` to
///   their Cargo.toml.
#[proc_macro_attribute]
pub fn metric(attr: TokenStream, item: TokenStream) -> TokenStream {
    match metric::metric(attr, item) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
