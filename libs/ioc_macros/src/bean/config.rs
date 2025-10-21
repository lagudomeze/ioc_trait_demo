use darling::{FromMeta, ast::NestedMeta};
use syn::Expr;

#[derive(Debug, FromMeta, PartialEq)]
pub(crate) struct Named {
    pub name: String,
    pub default: Option<Expr>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Config {
    Default,
    Trivial,
    Named(Named),
}

impl FromMeta for Config {
    fn from_none() -> Option<Self> {
        Some(Self::Default)
    }

    fn from_word() -> darling::Result<Self> {
        Ok(Self::Trivial)
    }

    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        Named::from_list(items).map(Config::Named)
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self::Named(Named {
            name: value.to_string(),
            default: None,
        }))
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{Attribute, parse_quote};

    #[test]
    fn test_named() {
        let attr: Attribute = parse_quote!( #[xxx(name = "test")] );
        let named = Named::from_meta(&attr.meta).unwrap();
        assert_eq!(
            named,
            Named {
                name: "test".to_string(),
                default: None,
            }
        );

        let attr: Attribute = parse_quote!( #[xxx(name = "test", default = 1 + 2)] );
        let named = Named::from_meta(&attr.meta).unwrap();
        assert_eq!(
            named,
            Named {
                name: "test".to_string(),
                default: Some(parse_quote!(1 + 2)),
            }
        );
    }
    #[test]
    fn test_config_none() {
        let config_meta: Option<Config> = Config::from_none();
        assert_eq!(config_meta, Some(Config::Default));
    }
    #[test]
    fn test_config_unnamed() {
        let attr: Attribute = parse_quote!( #[config] );
        let config_meta = Config::from_meta(&attr.meta).unwrap();
        assert_eq!(config_meta, Config::Trivial);
    }
    #[test]
    fn test_config_named() {
        let attr: Attribute = parse_quote!( #[config(name = "test")] );
        let config_meta = Config::from_meta(&attr.meta).unwrap();
        assert_eq!(
            config_meta,
            Config::Named(Named {
                name: "test".to_string(),
                default: None,
            })
        );
    }
    #[test]
    fn test_config_named2() {
        let attr: Attribute = parse_quote!( #[config = "test"] );
        let config_meta = Config::from_meta(&attr.meta).unwrap();
        assert_eq!(
            config_meta,
            Config::Named(Named {
                name: "test".to_string(),
                default: None,
            })
        );
    }
    #[test]
    fn test_config_named_with_default_value() {
        let attr: Attribute = parse_quote!( #[config(name = "test", default = 12)] );
        let config_meta = Config::from_meta(&attr.meta).unwrap();
        assert_eq!(
            config_meta,
            Config::Named(Named {
                name: "test".to_string(),
                default: Some(parse_quote!(12)),
            })
        );
    }
    #[test]
    fn test_config_named_with_default_value2() {
        let attr: Attribute = parse_quote!( #[config(name = "test", default = "1234")] );
        let config_meta = Config::from_meta(&attr.meta).unwrap();
        assert_eq!(
            config_meta,
            Config::Named(Named {
                name: "test".to_string(),
                default: Some(parse_quote!(1234)),
            })
        );
    }
}
