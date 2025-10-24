mod bean;
mod bind;
mod context;

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_attribute]
pub fn with(attr: TokenStream, item: TokenStream) -> TokenStream {
    let bind: bind::Bind = match syn::parse(attr) {
        Ok(v) => v,
        Err(e) => {
            return e.to_compile_error().into();
        }
    };

    let mut input = syn::parse_macro_input!(item as syn::ItemFn);

    bind.add_bounds(&mut input.sig.generics);

    input.into_token_stream().into()
}

#[proc_macro_derive(Bean, attributes(rivete))]
pub fn bean(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match bean::Bean::from_derive_input(&input) {
        Ok(bean_struct) => bean_struct.into_token_stream().into(),
        Err(err) => err.write_errors().into(),
    }
}

#[proc_macro_derive(Context, attributes(rivete))]
pub fn context(input: TokenStream) -> TokenStream {
    unimplemented!()
}
