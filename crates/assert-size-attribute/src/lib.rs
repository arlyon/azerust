use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, Type};

#[proc_macro_attribute]
pub fn assert_eq_size(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let attr = parse_macro_input!(attr as Type);
    let name = input.ident.clone();

    TokenStream::from(quote! {
        #input

        static_assertions::assert_eq_size!(
            #name,
            #attr
        );
    })
}
