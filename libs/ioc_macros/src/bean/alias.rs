use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

#[derive(Debug, FromMeta, PartialEq)]
pub(crate) struct Alias {
    name: syn::Path,
    ctx: Option<syn::Path>,
}

impl Alias {
    pub(crate) fn generate(
        &self,
        ty: &syn::Ident,
        ioc: &TokenStream,
    ) -> darling::Result<TokenStream> {
        let alias_name = &self.name;
        let ctx_name = if let Some(nane) = &self.ctx {
            nane
        } else {
            &parse_quote!( #ioc::prelude::Ctx )
        };
        Ok(quote! {
            impl Alias<#alias_name> for #ctx_name {
                type Key = #ty;
            }
        })
    }
}
