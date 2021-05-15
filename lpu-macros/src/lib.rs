extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Parse)]
pub fn parse_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    let gen = quote! {
        impl #name {
            pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) ->
                Result<Self> {
                Ok(ok!(Self::from_u8(next!(msg))))
            }
        }
    };

    gen.into()
}
