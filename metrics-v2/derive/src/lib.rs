// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use proc_macro::TokenStream;

mod metric;

/// Declare a global metric that can be accessed via the `metrics` method.
///
/// # Parameters
/// - (optional) `name`: The name that the metric should be exposed as. If not
///   specified then the default name is one based on the path to the metric
///   along with its name.
#[proc_macro_attribute]
pub fn metric(attr: TokenStream, item: TokenStream) -> TokenStream {
    match metric::metric(attr, item) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
