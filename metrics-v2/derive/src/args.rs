// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use quote::ToTokens;
use std::fmt::{Display, Formatter, Result};
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token};

#[derive(Clone)]
pub(crate) enum ArgName {
    Ident(Ident),
    Crate(Token![crate]),
}

impl Parse for ArgName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        Ok(match () {
            _ if lookahead.peek(Ident) => Self::Ident(input.parse()?),
            _ if lookahead.peek(Token![crate]) => Self::Crate(input.parse()?),
            _ => return Err(lookahead.error()),
        })
    }
}

impl ToTokens for ArgName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use self::ArgName::*;

        match self {
            Ident(ident) => ident.to_tokens(tokens),
            Crate(krate) => krate.to_tokens(tokens),
        }
    }
}

impl Display for ArgName {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use self::ArgName::*;

        match self {
            Ident(ident) => ident.fmt(f),
            Crate(_) => f.write_str("crate"),
        }
    }
}
