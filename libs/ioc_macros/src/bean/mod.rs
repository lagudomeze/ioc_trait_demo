mod config;
mod field;

use crate::bean::field::Field;
use darling::{Error, FromDeriveInput, Result, ast::Data, ast::Style};
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::Path;

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

#[derive(Debug, FromDeriveInput)]
pub(crate) struct Bean {
    /// The struct ident.
    ident: Ident,

    /// Receives the body of the struct or enum. We don't care about
    /// struct fields because we previously told darling we only accept structs.
    data: Data<(), Field>,

    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    ioc_crate: Option<Path>,
}

struct BuildInit<'a> {
    ident: &'a Ident,
    fields: &'a Data<(), Field>,
    ioc: &'a TokenStream,
}

impl BuildInit<'_> {
    fn generate(&self) -> Result<TokenStream> {
        let Self { ident, fields, ioc } = *self;

        if !fields.is_struct() {
            Err(Error::unsupported_shape("only struct is supported").with_span(ident))
        } else {
            let struct_fields = fields.as_ref().take_struct().expect("not here!");

            let field_initializers = struct_fields.iter().map(|f| f.as_init());

            let initializers = quote! {
                #(#field_initializers),*
            };

            let initializer = match struct_fields.style {
                Style::Tuple => {
                    quote! {
                        use ::#ioc::prelude::*;
                        #ident(
                            #initializers
                        )
                    }
                }
                Style::Struct => {
                    quote! {
                        use ::#ioc::prelude::*;
                        #ident{
                            #initializers
                        }
                    }
                }
                Style::Unit => {
                    quote! { #ident }
                }
            };
            Ok(quote! {
                 {
                     use ::#ioc::prelude::*;
                     #initializer
                 }
            })
        }
    }
}

impl Bean {
    pub(crate) fn generate(&self) -> Result<TokenStream> {
        let Self {
            ref ident,
            ref data,
            ref name,
            ref ioc_crate,
        } = *self;

        let ioc = resolve_ioc_crate(ioc_crate)?;

        let build_method = BuildInit {
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
                fn init_method(ctx: &mut InitCtx) -> #ioc::Result<()> {
                    let B = #build_method;
                    PLACE.initialize(ctx).write(B);
                    Ok(())
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

impl ToTokens for Bean {
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
    fn filed_config() {
        let input = r#"
            #[derive(Bean)]
            pub struct LogPatcher(
                #[config(name = "log.level", default = "info")]
                Handle<EnvFilter, Formatter>
            );
        "#;

        let parsed = parse_str(input).unwrap();
        let result = Bean::from_derive_input(&parsed);
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
        let result = Bean::from_derive_input(&parsed);
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
        let result = Bean::from_derive_input(&parsed);
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
        let result = Bean::from_derive_input(&parsed);
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
