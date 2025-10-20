mod bean;

use crate::bean::BeanSpecStruct;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_attribute]
pub fn bean(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    todo!()
}

#[proc_macro_derive(Bean, attributes(inject, bean))]
pub fn bean_definition(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match BeanSpecStruct::from_derive_input(&input) {
        Ok(bean_struct) => bean_struct.into_token_stream().into(),
        Err(err) => err.write_errors().into(),
    }
}
