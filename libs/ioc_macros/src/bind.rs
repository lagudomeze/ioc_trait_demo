use darling::FromMeta;
use syn::{Expr, Generics, Token, Type, TypeParamBound, parse_quote, punctuated::Punctuated};

#[derive(Debug, PartialEq, FromMeta)]
pub(crate) struct Bean {
    #[darling(default)]
    key: Option<syn::Path>,
    path: syn::Path,
}

impl Bean {
    pub(crate) fn add_bounds(&self, generics: &mut Generics) {
        let bean: Type = {
            let path = &self.path;
            syn::parse_quote! { #path }
        };
        let key = {
            if let Some(ref key) = self.key {
                syn::parse_quote! { #key }
            } else {
                bean.clone()
            }
        };

        let where_clause = generics
            .where_clause
            .get_or_insert_with(|| syn::WhereClause {
                where_token: Default::default(),
                predicates: Default::default(),
            });

        let predicate = syn::parse_quote! {
            Ctx : Registered<#key, Bean=#bean>
        };
        where_clause.predicates.push(predicate);
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Bounds(Punctuated<TypeParamBound, Token![+]>);
impl FromMeta for Bounds {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        let bounds: Punctuated<TypeParamBound, Token![+]> = parse_quote!( #expr );
        Ok(Self(bounds))
    }
}

#[derive(Debug, PartialEq, FromMeta)]
pub(crate) struct Trait {
    name: syn::Path,
    traits: Bounds,
    #[darling(default)]
    context: Option<syn::Path>,
}
impl Trait {
    pub(crate) fn add_bounds(&self, generics: &mut Generics) {
        let name: Type = {
            let name = &self.name;
            syn::parse_quote! { #name }
        };

        let trait_bound = &self.traits.0;

        let context: Type = if let Some(ref ctx_type) = self.context {
            syn::parse_quote! { #ctx_type }
        } else {
            syn::parse_quote! { C }
        };

        let where_clause = generics
            .where_clause
            .get_or_insert_with(|| syn::WhereClause {
                where_token: Default::default(),
                predicates: Default::default(),
            });

        where_clause.predicates.push(syn::parse_quote! {
            #context: Alias<#name>
        });

        where_clause.predicates.push(syn::parse_quote! {
            Ctx: Registered<<#context as Alias<#name>>::Key, Bean: #trait_bound>
        });
    }
}

#[derive(Debug, PartialEq, FromMeta)]
#[darling(derive_syn_parse)]
pub(crate) enum Bind {
    Bean(Bean),
    Alias(Trait),
}
impl Bind {
    pub(crate) fn add_bounds(&self, generics: &mut Generics) {
        match self {
            Bind::Bean(bean) => bean.add_bounds(generics),
            Bind::Alias(trait_) => trait_.add_bounds(generics),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{Attribute, parse_quote};

    #[test]
    fn teest() {
        let value = quote::quote! { A };

        println!("{value:#?}");

        let bounds: Punctuated<TypeParamBound, Token![+]> = parse_quote!( #value );

        println!("{bounds:#?}");
    }

    #[test]
    fn test_bind() {
        let bean: Attribute = parse_quote!( #[with(bean = B, key = BKey)] );

        let bean = Bean::from_meta(&bean.meta).unwrap();
        assert_eq!(
            bean,
            Bean {
                path: parse_quote!(B),
                key: Some(parse_quote!(BKey)),
            }
        );
    }

    #[test]
    fn test_trait() {
        let bean: Attribute = parse_quote!( #[with(name = AKey, traits = A)] );

        let bean = Bean::from_meta(&bean.meta).unwrap();
        assert_eq!(
            bean,
            Bean {
                path: parse_quote!(B),
                key: Some(parse_quote!(BKey)),
            }
        );
    }
}
