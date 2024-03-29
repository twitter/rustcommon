// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::args::ArgName;
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::FoundCrate;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_quote, Error, Expr, Ident, ItemStatic, Path, Token};

struct SingleArg<T> {
    ident: ArgName,
    eq: Token![=],
    value: T,
}

impl<T: Parse> Parse for SingleArg<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl<T: ToTokens> ToTokens for SingleArg<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.eq.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}

#[derive(Default)]
struct MetricArgs {
    name: Option<SingleArg<Expr>>,
    namespace: Option<SingleArg<Expr>>,
    description: Option<SingleArg<Expr>>,
    krate: Option<SingleArg<Path>>,
}

impl Parse for MetricArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = MetricArgs::default();
        let mut first = true;

        fn duplicate_arg_error(
            span: Span,
            arg: &impl std::fmt::Display,
        ) -> syn::Result<MetricArgs> {
            Err(Error::new(
                span,
                format!("Unexpected duplicate argument '{}'", arg),
            ))
        }

        while !input.is_empty() {
            if !first {
                let _: Token![,] = input.parse()?;
            }
            first = false;

            let arg: ArgName = input.fork().parse()?;
            match &*arg.to_string() {
                "name" => {
                    let name = input.parse()?;
                    match args.name {
                        None => args.name = Some(name),
                        Some(_) => return duplicate_arg_error(name.span(), &arg),
                    }
                }
                "namespace" => {
                    let namespace = input.parse()?;
                    match args.namespace {
                        None => args.namespace = Some(namespace),
                        Some(_) => return duplicate_arg_error(namespace.span(), &arg),
                    }
                }
                "description" => {
                    let description = input.parse()?;
                    match args.description {
                        None => args.description = Some(description),
                        Some(_) => return duplicate_arg_error(description.span(), &arg),
                    }
                }
                "crate" => {
                    let krate = SingleArg {
                        ident: input.parse()?,
                        eq: input.parse()?,
                        value: Path::parse_mod_style(input)?,
                    };
                    match args.krate {
                        None => args.krate = Some(krate),
                        Some(_) => return duplicate_arg_error(krate.span(), &arg),
                    }
                }
                x => {
                    return Err(Error::new(
                        arg.span(),
                        format!("Unrecognized argument '{}'", x),
                    ))
                }
            }
        }

        Ok(args)
    }
}

pub(crate) fn metric(
    attr_: proc_macro::TokenStream,
    item_: proc_macro::TokenStream,
) -> syn::Result<TokenStream> {
    let mut item: ItemStatic = syn::parse(item_)?;
    let args: MetricArgs = syn::parse(attr_)?;

    let krate: TokenStream = match args.krate {
        Some(krate) => krate.value.to_token_stream(),
        None => proc_macro_crate::crate_name("rustcommon-metrics")
            .map(|krate| match krate {
                FoundCrate::Name(name) => {
                    assert_ne!(name, "");
                    Ident::new(&name, Span::call_site()).to_token_stream()
                }
                FoundCrate::Itself => quote! { rustcommon_metrics },
            })
            .unwrap_or(quote! { rustcommon_metrics }),
    };

    let name: TokenStream = match args.name {
        Some(name) => name.value.to_token_stream(),
        None => {
            let item_name = &item.ident;
            quote! { concat!(module_path!(), "::", stringify!(#item_name)) }
        }
    };

    let namespace: TokenStream = match args.namespace {
        Some(namespace) => namespace.value.to_token_stream(),
        None => {
            quote! {""}
        }
    };

    let description: TokenStream = match args.description {
        Some(description) => description.value.to_token_stream(),
        None => {
            quote! {""}
        }
    };

    let static_name = &item.ident;
    let static_expr = &item.expr;
    let static_type = &item.ty;

    item.expr = Box::new(parse_quote! {{
        // Rustc reserves attributes that start with "rustc". Since rustcommon
        // starts with "rustc" we can't use it directly within attributes. To
        // work around this, we first import the exports submodule and then use
        // that for the attributes.
        use #krate::export;

        #[export::linkme::distributed_slice(export::METRICS)]
        #[linkme(crate = export::linkme)]
        static __: #krate::MetricEntry = #krate::MetricEntry::_new_const(
            #krate::MetricWrapper(&#static_name.metric),
            #static_name.name(),
            #namespace,
            #description
        );

        #krate::MetricInstance::new(#static_expr, #name, #description)
    }});
    item.ty = Box::new(parse_quote! { #krate::MetricInstance<#static_type> });

    Ok(quote! { #item })
}
