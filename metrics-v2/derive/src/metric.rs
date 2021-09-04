// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use proc_macro_crate::FoundCrate;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_quote, Error, Expr, Ident, ItemStatic, Path, Token};

struct SingleArg<T, I = Ident> {
    ident: I,
    eq: Token![=],
    value: T,
}

impl<T: Parse, I: Parse> Parse for SingleArg<T, I> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl<T: ToTokens, I: ToTokens> ToTokens for SingleArg<T, I> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.eq.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}

#[derive(Default)]
struct MetricArgs {
    name: Option<SingleArg<Expr>>,
    krate: Option<SingleArg<Path, Token![crate]>>,
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

            let lookahead = input.lookahead1();
            let (arg, arg_span): (Cow<str>, _) = if lookahead.peek(Ident) {
                let ident: Ident = input.fork().parse()?;
                (Cow::Owned(ident.to_string()), ident.span())
            } else if lookahead.peek(Token![crate]) {
                (
                    Cow::Borrowed("crate"),
                    input.fork().parse::<Token![crate]>()?.span(),
                )
            } else {
                return Err(lookahead.error());
            };

            match &*arg {
                "name" => {
                    let name: SingleArg<Expr> = input.parse()?;
                    if args.name.is_some() {
                        return duplicate_arg_error(name.span(), &arg);
                    }
                    args.name = Some(name);
                }
                "crate" => {
                    let krate: SingleArg<Path, Token![crate]> = SingleArg {
                        ident: input.parse()?,
                        eq: input.parse()?,
                        value: Path::parse_mod_style(input)?,
                    };
                    if args.krate.is_some() {
                        return duplicate_arg_error(krate.span(), &arg);
                    }
                    args.krate = Some(krate);
                }
                x => {
                    return Err(Error::new(
                        arg_span,
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
        None => proc_macro_crate::crate_name("rustcommon_metrics_v2")
            .map(|krate| match krate {
                FoundCrate::Name(name) => Ident::new(&name, Span::call_site()).to_token_stream(),
                FoundCrate::Itself => quote! { crate },
            })
            .unwrap_or(quote! { rustcommon_metrics_v2 }),
    };

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
        // Rustc reserves attributes that start with "rustc". Since rustcommon
        // starts with "rustc" we can't use it directly within attributes. To
        // work around this, we first import the exports submodule and then use
        // that for the attributes.
        use #krate::export;

        #[export::linkme::distributed_slice(export::METRICS)]
        #[linkme(crate = export::linkme)]
        static __: #krate::MetricEntry = #krate::MetricEntry {
            name: #name,
            metric: &#static_name
        };

        #static_expr
    }};

    item.expr = Box::new(new_expr);

    Ok(quote! { #item })
}
