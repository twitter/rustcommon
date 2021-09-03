// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use proc_macro2::{Span, TokenStream};
use proc_macro_crate::FoundCrate;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_quote, Error, Expr, Ident, ItemStatic, Token};

struct SingleArg<T> {
    ident: Ident,
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
}

impl Parse for MetricArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = MetricArgs::default();

        while !input.is_empty() {
            let name: Ident = input.fork().parse()?;

            match &*name.to_string() {
                "name" => {
                    let name: SingleArg<Expr> = input.parse()?;
                    if args.name.is_some() {
                        return Err(Error::new(
                            name.span(),
                            format!("Duplicate argument '{}'", name.ident),
                        ));
                    }
                    args.name = Some(name);
                }
                x => {
                    return Err(Error::new(
                        name.span(),
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

    let krate: TokenStream = proc_macro_crate::crate_name("metrics_v2")
        .map(|krate| match krate {
            FoundCrate::Name(name) => Ident::new(&name, Span::call_site()).to_token_stream(),
            FoundCrate::Itself => quote! { crate },
        })
        .unwrap_or(quote! { rustcommon_metrics_v2 });

    let name: TokenStream = match args.name {
        Some(name) => name.value.to_token_stream(),
        None => {
            let item_name = &item.ident;
            quote! { concat!(module_path!(), "::", stringify!(#item_name)) }
        }
    };

    let static_name = &item.ident;
    let static_expr = &item.expr;
    let new_expr = parse_quote! {{
        // Since the crate name starts with "rustc" we can't use it as part of
        // an attribute. To work around that we need to use it here.
        use #krate::export;

        #[export::linkme::distributed_slice(#krate::export::METRICS)]
        #[linkme(crate = export::linkme)]
        static __: #krate::MetricEntry = #krate::MetricEntry::_new_const(
            #krate::MetricWrapper(&#static_name),
            #name
        );

        #static_expr
    }};

    item.expr = Box::new(new_expr);

    Ok(quote! { #item })
}
