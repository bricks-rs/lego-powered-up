// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Parse)]
pub fn parse_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    let trace_msg = format!("PARSE {}: {{}}", name);
    let gen = quote! {
        impl #name {
            pub fn parse<'a>(mut msg: impl Iterator<Item = &'a u8>) ->
                Result<Self> {
                let val = next!(msg);
                log::trace!(#trace_msg, val);
                Ok(ok!(Self::from_u8(val)))
            }
        }
    };

    gen.into()
}
