use proc_macro::TokenStream;

mod metric;

#[proc_macro_attribute]
pub fn metric(attr: TokenStream, item: TokenStream) -> TokenStream {
    match metric::metric(attr, item) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into()
    }
}
