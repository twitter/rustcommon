// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use proc_macro::TokenStream;
use quote::quote;
use syn::Ident;

mod args;
mod metric;

/// Declare a global metric that can be accessed via the `metrics` method.
///
/// Note that this will change the type of the generated static to be
/// `MetricInstance<MetricTy>`. It implements both [`Deref`] and [`DerefMut`]
/// so it can be used much the same as a normal static.  
///
/// # Parameters
/// - (optional) `name`: The string name that the metric should be exposed as.
///   If not specified then the default name is one based on the path to the
///   metric along with its name.
/// - (optional) `crate`: The path to the `rustcommon_metrics` crate. This
///   allows the `metric` macro to be used within other macros that get exported
///   to third-party crates which may not have added `rustcommon_metrics` to
///   their Cargo.toml.
/// - (optional) `description`: A textual description of the metric.
///   If not specified, or specified as a blank string then defaults to None
///
/// [`Deref`]: std::ops::Deref
/// [`DerefMut`]: std::ops::DerefMut
#[proc_macro_attribute]
pub fn metric(attr: TokenStream, item: TokenStream) -> TokenStream {
    match metric::metric(attr, item) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// This macro statically converts an ident to a lowercased string
/// at compile time.
///
/// In the future this could be replaced with some const code. However,
/// `std::str::from_utf8_unchecked` is not stably const just yet so we
/// need this macro as a workaround.
#[proc_macro]
pub fn to_lowercase(input: TokenStream) -> TokenStream {
    let ident = syn::parse_macro_input!(input as Ident);
    let name = ident.to_string().to_ascii_lowercase();
    let literal = syn::LitStr::new(&name, ident.span());
    let tokens = quote! { #literal };

    tokens.into()
}
