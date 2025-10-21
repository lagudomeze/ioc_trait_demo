use crate::bean::config::{Config, Named};
use darling::{Error, FromField};
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use syn::{Expr, Type};

#[derive(Debug, FromField, PartialEq)]
#[darling(attributes(rivete), and_then = Self::validate)]
pub(crate) struct Field {
    ty: Type,
    ident: Option<Ident>,
    config: Config,
}

impl Field {
    fn validate(self) -> darling::Result<Self> {
        if self.ident.is_none() && self.config == Config::Trivial {
            return Err(Error::custom(
                "Trivial config cannot be used for tuple struct fields! You must provide a name for the config field.",
            ));
        }
        Ok(self)
    }

    pub(crate) fn as_init(&self) -> FieldInit<'_> {
        FieldInit {
            field: self,
        }
    }
}

pub(crate) struct FieldInit<'a> {
    field: &'a Field,
}

impl ToTokens for FieldInit<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Field {
            ty: _ty,
            ident,
            config,
        } = self.field;

        let initializer = match config {
            Config::Default => quote! { ::core::default::Default::default() },
            Config::Trivial => quote! { ctx.get_config::<_>(#ident)? },
            Config::Named(Named { name, default }) => {
                if let Some(value) = default {
                    match value {
                        Expr::Lit(lit) => {
                            quote! { ctx.get_config_or::<_>(#name, #lit.into())? }
                        }
                        other => {
                            quote! { ctx.get_config_or::<_>(#name, #other)? }
                        }
                    }
                } else {
                    quote! { ctx.get_config::<_>(#name)?}
                }
            }
        };

        if let Some(field_name) = ident {
            tokens.extend(quote! { #field_name : #initializer })
        } else {
            tokens.extend(initializer)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn filed_config() -> Result<(), String> {
        let raw = parse_quote!(
            #[rivete(config(name = "log.level", default = "info"))]
            Handle<EnvFilter, Formatter>
        );
        let field = Field::from_field(&raw).map_err(|err| err.to_string())?;

        if let Config::Named(named) = &field.config {
            assert_eq!(named.name, "log.level");
            assert_eq!(named.default, Some(parse_quote!(info)));
        } else {
            return Err("Expected Config::Named variant".to_string());
        }
        Ok(())
    }

    #[test]
    fn filed_config_name() -> Result<(), String> {
        let raw = parse_quote!(
            #[rivete(config = "log.level")]
            Handle<EnvFilter, Formatter>
        );
        let field = Field::from_field(&raw).map_err(|err| err.to_string())?;
        if let Config::Named(named) = &field.config {
            assert_eq!(named.name, "log.level");
            assert_eq!(named.default, None);
        } else {
            return Err("Expected Config::Named variant".to_string());
        }
        Ok(())
    }

    #[test]
    fn filed_config_name_fail() -> Result<(), String> {
        let field = parse_quote!(
            #[rivete(config)]
            Handle<EnvFilter, Formatter>
        );
        if let Err(err) = Field::from_field(&field) {
            assert_eq!(
                err.to_string(),
                "Trivial config cannot be used for tuple struct fields! You must provide a name for the config field."
            );
        } else {
            return Err("Expected tuple struct fields".to_string());
        }
        Ok(())
    }

    #[test]
    fn filed_config_name_struct() -> Result<(), String> {
        let field = Field::from_field(&parse_quote!(
            #[rivete(config)]
            test: Handle<EnvFilter, Formatter>
        ))
        .map_err(|err| err.to_string())?;

        assert_eq!(field.config, Config::Trivial);

        Ok(())
    }
}
