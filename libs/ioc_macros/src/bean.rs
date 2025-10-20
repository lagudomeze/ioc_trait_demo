use darling::{Error, FromDeriveInput, FromField, Result, ast::Data, ast::Style};
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{Expr, Path, Type};

use crate::bean::meta::ConfigMeta;

pub(crate) fn resolve_ioc_crate(ioc_crate: &Option<Path>) -> Result<TokenStream> {
    if let Some(ioc_crate) = ioc_crate {
        Ok(quote! { #ioc_crate })
    } else {
        use proc_macro_crate::{FoundCrate, crate_name};
        match crate_name("ioc") {
            Ok(FoundCrate::Itself) => Ok(quote! { crate }),
            Ok(FoundCrate::Name(name)) => {
                let ident = format_ident!("{}", name);
                Ok(quote! { #ident })
            }
            Err(err) => Err(Error::custom(err)),
        }
    }
}

mod meta {
    use darling::util::path_to_string;
    use darling::{Error, FromMeta, Result, ast::NestedMeta};
    use syn::{Expr, Meta, Path};

    #[derive(Debug, PartialEq)]
    pub(crate) enum ConfigMeta {
        Trivial,
        Named { name: String, default: Option<Expr> },
    }

    impl FromMeta for ConfigMeta {
        fn from_word() -> Result<Self> {
            Ok(Self::Trivial)
        }

        fn from_list(items: &[NestedMeta]) -> Result<Self> {
            match items.len() {
                0 => dbg!(Self::from_word()),
                1 | 2 => {
                    let mut errors = Error::accumulator();
                    let mut name: Option<String> = None;
                    let mut default: Option<Expr> = None;
                    for item in items {
                        match item {
                            NestedMeta::Meta(kv) => {
                                match path_to_string(kv.path()).as_str() {
                                    "name" => {
                                        if name.is_some() {
                                            errors.push(
                                                Error::duplicate_field("name").with_span(item),
                                            );
                                        } else {
                                            name = errors.handle(String::from_meta(kv));
                                        }
                                    }
                                    "default" => {
                                        if default.is_some() {
                                            errors.push(
                                                Error::duplicate_field("default").with_span(item),
                                            );
                                        } else {
                                            match kv {
                                                Meta::NameValue(value) => {
                                                    default = Some(value.value.clone());
                                                }
                                                Meta::List(list) => errors.push(
                                                    Error::unexpected_type("meta_list")
                                                        .with_span(list),
                                                ),
                                                Meta::Path(path) => errors.push(
                                                    Error::unexpected_type("path").with_span(path),
                                                ),
                                            };
                                        }
                                    }
                                    other => errors.push(
                                        Error::unknown_field_with_alts(other, &["name", "default"])
                                            .with_span(item),
                                    ),
                                }
                                if name.is_some() {}
                            }
                            NestedMeta::Lit(lit) => {
                                return Err(Error::unexpected_lit_type(lit));
                            }
                        }
                    }
                    if let Some(name) = name {
                        errors.finish()?;
                        Ok(Self::Named { name, default })
                    } else {
                        errors.push(Error::missing_field("name"));
                        errors.finish()?;
                        unreachable!()
                    }
                }
                other => Err(Error::too_many_items(other)),
            }
        }

        fn from_string(value: &str) -> Result<Self> {
            Ok(Self::Named {
                name: value.to_string(),
                default: None,
            })
        }
    }

    #[derive(Debug, PartialEq)]
    pub(crate) enum BeanMeta {
        Trivial,
        Spec { spec: Path },
    }

    impl FromMeta for BeanMeta {
        fn from_word() -> Result<Self> {
            Ok(Self::Trivial)
        }

        fn from_list(items: &[NestedMeta]) -> Result<Self> {
            match items.len() {
                0 => Self::from_word(),
                1 => match items[0] {
                    NestedMeta::Meta(Meta::Path(_)) => Self::from_word(),
                    NestedMeta::Meta(Meta::NameValue(ref value)) => Self::from_expr(&value.value),
                    NestedMeta::Meta(Meta::List(_)) => {
                        #[derive(Debug, FromMeta)]
                        struct BeanParam {
                            spec: Path,
                        }
                        let param = BeanParam::from_list(items)?;
                        Ok(Self::Spec { spec: param.spec })
                    }
                    NestedMeta::Lit(ref lit) => Err(Error::unexpected_lit_type(lit)),
                },
                other => Err(Error::too_many_items(other)),
            }
        }

        fn from_expr(expr: &Expr) -> Result<Self> {
            match *expr {
                Expr::Group(ref group) => Self::from_expr(&group.expr),
                Expr::Path(ref path) => Ok(Self::Spec {
                    spec: path.path.clone(),
                }),
                _ => Err(Error::unexpected_expr_type(expr)),
            }
            .map_err(|e| e.with_span(expr))
        }
    }

    #[cfg(test)]
    mod test {
        use darling::{FromField, FromMeta};
        use syn::{Attribute, parse_quote};

        use crate::bean::BeanField;
        use crate::bean::meta::{BeanMeta, ConfigMeta};

        #[test]
        fn test_config_meta_none() {
            let config_meta: Option<ConfigMeta> = ConfigMeta::from_none();
            assert_eq!(config_meta, None);
        }

        #[test]
        fn test_config_meta_unnamed() {
            let attr: Attribute = parse_quote!( #[config] );
            let config_meta = ConfigMeta::from_meta(&attr.meta).unwrap();
            assert_eq!(config_meta, ConfigMeta::Trivial);
        }

        #[test]
        fn test_config_meta_named() {
            let attr: Attribute = parse_quote!( #[config(name = "test")] );
            let config_meta = ConfigMeta::from_meta(&attr.meta).unwrap();
            assert_eq!(
                config_meta,
                ConfigMeta::Named {
                    name: "test".to_string(),
                    default: None,
                }
            );
        }

        #[test]
        fn test_config_meta_named2() {
            let attr: Attribute = parse_quote!( #[config = "test"] );
            let config_meta = ConfigMeta::from_meta(&attr.meta).unwrap();
            assert_eq!(
                config_meta,
                ConfigMeta::Named {
                    name: "test".to_string(),
                    default: None,
                }
            );
        }

        #[test]
        fn test_config_meta_named_with_default_value() {
            let attr: Attribute = parse_quote!( #[config(name = "test", default = 12)] );
            let config_meta = ConfigMeta::from_meta(&attr.meta).unwrap();
            assert_eq!(
                config_meta,
                ConfigMeta::Named {
                    name: "test".to_string(),
                    default: Some(parse_quote!(12)),
                }
            );
        }

        #[test]
        fn test() {
            let field = parse_quote!(
                #[inject(config(name = "web.static.path", default = "static"))]
                test: string
            );

            let bean_field = BeanField::from_field(&field).unwrap();

            println!("{:?}", bean_field.config);
        }

        #[test]
        fn test_config_meta_named_with_default_value_str() {
            let attr: Attribute =
                parse_quote!( #[config(name = "web.static.path", default = "static")] );

            let config_meta = ConfigMeta::from_meta(&attr.meta).unwrap();
            assert_eq!(
                config_meta,
                ConfigMeta::Named {
                    name: "web.static.path".to_string(),
                    default: Some(parse_quote!("static")),
                }
            );
        }

        #[test]
        fn test_bean_meta() {
            assert_eq!(BeanMeta::from_none(), None);

            let attr: Attribute = parse_quote!( #[bean] );
            let meta = BeanMeta::from_meta(&attr.meta).unwrap();
            assert_eq!(meta, BeanMeta::Trivial);

            let attr: Attribute = parse_quote!( #[bean = aa::bb::Cc] );
            let meta = BeanMeta::from_meta(&attr.meta).unwrap();
            assert_eq!(
                meta,
                BeanMeta::Spec {
                    spec: parse_quote!(aa::bb::Cc),
                }
            );

            let attr: Attribute = parse_quote!( #[bean(spec = aa::bb::Cc)] );
            let meta = BeanMeta::from_meta(&attr.meta).unwrap();
            assert_eq!(
                meta,
                BeanMeta::Spec {
                    spec: parse_quote!(aa::bb::Cc),
                }
            );
        }
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(inject))]
pub struct BeanField {
    ty: Type,
    ident: Option<Ident>,
    #[darling(default)]
    config: Option<ConfigMeta>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(bean))]
pub(crate) struct BeanSpecStruct {
    /// The struct ident.
    ident: Ident,

    /// Receives the body of the struct or enum. We don't care about
    /// struct fields because we previously told darling we only accept structs.
    data: Data<(), BeanField>,

    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    ioc_crate: Option<Path>,
}

struct FieldInitializer<'a>(&'a BeanField);

impl<'a> From<&'a BeanField> for FieldInitializer<'a> {
    fn from(value: &'a BeanField) -> Self {
        Self(value)
    }
}

impl ToTokens for FieldInitializer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let BeanField {
            ty: _ty,
            ident,
            config,
        } = self.0;

        let initializer = if let Some(config) = config {
            match config {
                ConfigMeta::Trivial => quote! { ctx.get_config::<_>(#ident)? },
                ConfigMeta::Named { name, default } => {
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
            }
        } else {
            quote! { Default::default() }
        };

        if let Some(field_name) = ident {
            tokens.extend(quote! { #field_name : #initializer })
        } else {
            tokens.extend(initializer)
        }
    }
}

struct BuildMethod<'a> {
    ident: &'a Ident,
    fields: &'a Data<(), BeanField>,
    ioc: &'a TokenStream,
}

impl BuildMethod<'_> {
    fn generate(&self) -> Result<TokenStream> {
        let Self { ident, fields, ioc } = *self;

        if !fields.is_struct() {
            Err(Error::unsupported_shape("only struct is supported").with_span(ident))
        } else {
            let struct_fields = fields.as_ref().take_struct().expect("not here!");

            let field_initializers = struct_fields.iter().cloned().map(FieldInitializer::from);

            let initializers = quote! {
                #(#field_initializers),*
            };

            let initializer = match struct_fields.style {
                Style::Tuple => {
                    quote! { #ident(
                        #initializers
                    ) }
                }
                Style::Struct => {
                    quote! { #ident{
                        #initializers
                    } }
                }
                Style::Unit => {
                    quote! { #ident }
                }
            };
            Ok(quote! {
                 {
                     #initializer
                 }
            })
        }
    }
}

impl BeanSpecStruct {
    pub(crate) fn generate(&self) -> Result<TokenStream> {
        let Self {
            ref ident,
            ref data,
            ref name,
            ref ioc_crate,
        } = *self;

        let ioc = resolve_ioc_crate(ioc_crate)?;

        let build_method = BuildMethod {
            ident,
            fields: data,
            ioc: &ioc,
        };

        let build_method = build_method.generate()?;

        let key = if let Some(key) = name {
            quote! { #key }
        } else {
            quote! { #ident }
        };

        let mod_ident = format_ident!("{}_bean_register", key.to_string().to_lowercase());

        Ok(quote! {
            pub mod #mod_ident {
                use ::#ioc::prelude::*;
                use ::linkme::distributed_slice;
                use super::#ident;

                static PLACE: StaticPlace<#ident> = StaticPlace::uninit();

                #[distributed_slice(INIT_METHODS)]
                static INIT_METHOD: InitMethod = init_method;

                #[inline]
                fn init_method(phase: &mut InitPhase) {
                    let B = #build_method;
                    PLACE.initialize(phase).write(B);
                }

                #[distributed_slice(DROP_METHODS)]
                static DROP_METHOD: DropMethod = drop_method;

                #[inline]
                fn drop_method(phase: &mut ActivePhase) {
                    unsafe {
                        PLACE.deinitialize(phase)
                    }
                }

                unsafe impl Registered<#key> for Ctx {
                    type Bean = #ident;

                    #[inline(always)]
                    fn get(ctx: &Ctx) -> &Self::Bean {
                        PLACE.get(ctx)
                    }

                    #[inline(always)]
                    fn get_mut(ctx: &mut Ctx) -> &mut Self::Bean {
                        PLACE.get_mut(ctx)
                    }
                }
            }
        })
    }
}

impl ToTokens for BeanSpecStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.generate() {
            Ok(tt) => {
                tokens.extend(tt);
            }
            Err(err) => {
                tokens.extend(err.write_errors());
            }
        }
    }
}

#[cfg(test)]
mod test {
    use syn::{parse_quote, parse_str};

    use super::*;

    #[test]
    fn it_works() {
        let input = r#"
            #[derive(Bean)]
            #[bean(ioc_crate = "ioc")]
            pub struct LogPatcher(
                #[inject(default)]
                Handle<EnvFilter, Formatter>
            );
        "#;

        let parsed = parse_str(input).unwrap();
        let result = BeanSpecStruct::from_derive_input(&parsed);
        if let Err(err) = result {
            println!("err 0:{}", err.write_errors().to_string());
            return;
        }
        let bean_struct = result.unwrap();

        if let Err(err) = bean_struct.generate() {
            println!("err 1:{}", err.write_errors().to_string());
            return;
        }

        let file: syn::File = parse_quote!( #bean_struct);

        println!("{}", prettyplease::unparse(&file));
    }

    #[test]
    fn construct() {
        let input = r#"
            #[derive(Bean)]
            #[bean(ioc_crate = "ioc", construct = "Init")]
            pub struct LogPatcher(
                #[inject(default)]
                Handle<EnvFilter, Formatter>
            );
        "#;

        let parsed = parse_str(input).unwrap();
        let result = BeanSpecStruct::from_derive_input(&parsed);
        if let Err(err) = result {
            println!("err 0:{}", err.write_errors().to_string());
            return;
        }
        let bean_struct = result.unwrap();

        if let Err(err) = bean_struct.generate() {
            println!("err 1:{}", err.write_errors().to_string());
            return;
        }

        let file: syn::File = parse_quote!( #bean_struct);

        println!("{}", prettyplease::unparse(&file));
    }

    #[test]
    fn test_inject_config() {
        let input = r#"
            #[derive(Bean)]
            #[bean(ioc_crate = "ioc")]
            pub struct WebConfig {
                #[inject(config = "web.addr")]
                addr: String,
                #[inject(config = "web.graceful_shutdown_timeout")]
                shutdown_timeout: Duration,
                #[inject(config = "web.tracing")]
                tracing: bool,
            }
        "#;

        let parsed = parse_str(input).unwrap();
        let result = BeanSpecStruct::from_derive_input(&parsed);
        if let Err(err) = result {
            println!("err 0:{}", err.write_errors().to_string());
            return;
        }
        let bean_struct = result.unwrap();

        if let Err(err) = bean_struct.generate() {
            println!("err 1:{}", err.write_errors().to_string());
            return;
        }

        let file: syn::File = parse_quote!( #bean_struct);

        println!("{}", prettyplease::unparse(&file));
    }
}
