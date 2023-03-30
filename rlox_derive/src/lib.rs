use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn syntax_node(attr: TokenStream, input: TokenStream) -> TokenStream {
    rlox_derive_impl::syntax_node(attr.into(), input.into()).into()
}
