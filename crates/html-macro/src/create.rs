use crate::config::{HtmlConfig, ParsedHtmlConfigField};
use crate::maybe_dollar::PathAndMaybeDollar;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Token};

/// Parses the [`crate::define_html_macro`].
///
/// ```
/// # use html_macro::define_html_macro;
/// # use percy_dom::{VirtualNode, VirtualElement, VirtualText, IterableNodes};
/// # struct SomeDomType;
///
/// define_html_macro! {
///     my_html!
///     real_dom = (),
///     calls = html_macro::html_with_config,
/// };
/// ```
pub(super) struct CreateHtmlMacro {
    /// The name of the generated macro, such as the `my_html` in `macro_rules my_html!`.
    name: syn::Ident,
    /// The documentation for the generated macro.
    doc: Vec<LitStr>,
    /// The path to the [`crate::html_with_config`] macro.
    html_macro_path: Option<PathAndMaybeDollar>,
    html_config: HtmlConfig,
}

impl Parse for CreateHtmlMacro {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut doc = Vec::new();

        while let Ok(_) = input.parse::<Token![#]>() {
            let content;
            syn::bracketed!(content in input);

            match content.parse::<syn::Ident>()?.to_string().as_str() {
                "doc" => {
                    content.parse::<Token![=]>()?;
                    let doc_line = content.parse::<LitStr>()?;
                    doc.push(doc_line);
                }
                attrib => {
                    todo!("return error indicating that {attrib} is an unsupported attribute")
                }
            }
        }

        let name = input.parse::<syn::Ident>()?;
        input.parse::<Token![!]>()?;

        let mut html_macro_path = None;
        let mut html_config_fields = Vec::new();

        loop {
            if input.is_empty() {
                break;
            }

            let parse_field = input.fork();

            match input.parse::<syn::Ident>()?.to_string().as_str() {
                "calls" => {
                    input.parse::<Token![=]>()?;
                    html_macro_path = Some(input.parse::<PathAndMaybeDollar>()?);
                }
                _ => {
                    let field = ParsedHtmlConfigField::parse(&parse_field)?;
                    html_config_fields.push(field);
                    input.advance_to(&parse_field);
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let html_config = HtmlConfig::from_fields(html_config_fields);

        Ok(CreateHtmlMacro {
            name,
            doc,
            html_macro_path,
            html_config,
        })
    }
}

impl CreateHtmlMacro {
    /// Generate the tokens.
    ///
    /// Example generated tokens:
    /// ```
    /// # use html_macro::html_with_config;
    /// macro_rules! my_html {
    ///     ($(tokens:tt)*) => {
    ///         html_with_config! {
    ///             real_dom = SomeType ;
    ///             $(tokens)*
    ///         }
    ///     }
    ///  }
    /// ```
    pub fn into_tokens(self) -> TokenStream {
        let CreateHtmlMacro {
            name,
            doc,
            html_macro_path,
            html_config: config,
        } = self;
        let HtmlConfig { real_dom_ty } = config;

        let doc = doc.into_iter().map(|doc_line| {
            quote! {
                #[doc = #doc_line]
            }
        });

        let html_macro_path = html_macro_path.unwrap_or_else(|| {
            syn::parse2(quote! {
                html_with_config
            })
            .unwrap()
        });

        quote! {
            #(#doc)*
            #[macro_export]
            macro_rules! #name {
                ($($tokens:tt)*) => {
                    #html_macro_path! {
                        real_dom = #real_dom_ty,
                        ;
                        $($tokens)*
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    /// Verify that we can create an `html!` macro with the given `RealDom`.
    #[test]
    fn define_html_macro_with_real_dom() {
        let tokens = quote! {
            my_html!
            real_dom = SomeType
        };
        let expected = quote! {
            #[macro_export]
            macro_rules! my_html {
                ($($tokens:tt)*) => {
                    html_with_config! {
                        real_dom = SomeType,
                        ;
                        $($tokens)*
                    }
                }
            }
        };
        assert_expected_create_html_tokens(tokens, expected);
    }

    /// Verify that the list of configuration supports a trailing comma.
    #[test]
    fn allows_trailing_comma() {
        let tokens = quote! {
            my_html!
            real_dom = SomeType,
        };
        let expected = quote! {
            #[macro_export]
            macro_rules! my_html {
                ($($tokens:tt)*) => {
                    html_with_config! {
                        real_dom = SomeType,
                        ;
                        $($tokens)*
                    }
                }
            }
        };
        assert_expected_create_html_tokens(tokens, expected);
    }

    /// Verify that we can configure the path to the [`crate::define_html_macro`].
    #[test]
    fn configure_html_macro_path() {
        let tokens = quote! {
            my_html!
            real_dom = SomeType,
            calls = path::to::html_with_config
        };
        let expected = quote! {
            #[macro_export]
            macro_rules! my_html {
                ($($tokens:tt)*) => {
                    path::to::html_with_config! {
                        real_dom = SomeType,
                        ;
                        $($tokens)*
                    }
                }
            }
        };
        assert_expected_create_html_tokens(tokens, expected);
    }

    /// Verify that we can provide documentation for the generated macro.
    #[test]
    fn sets_documentation() {
        let tokens = quote! {
            /// Hello world.
            /// Multiple lines work.
            my_html!
            real_dom = SomeType,
        };
        let expected = quote! {
            /// Hello world.
            /// Multiple lines work.
            #[macro_export]
            macro_rules! my_html {
                ($($tokens:tt)*) => {
                    html_with_config! {
                        real_dom = SomeType,
                        ;
                        $($tokens)*
                    }
                }
            }
        };
        assert_expected_create_html_tokens(tokens, expected);
    }

    /// Verify that we can use `$crate` in paths to the `real_dom`.
    #[test]
    fn supports_dollar_crate() {
        let tokens = quote! {
            my_html!
            real_dom = $crate::path::to::SomeType,
            calls = $crate::path::to::html_with_config
        };
        let expected = quote! {
            #[macro_export]
            macro_rules! my_html {
                ($($tokens:tt)*) => {
                    $crate::path::to::html_with_config! {
                        real_dom = $crate::path::to::SomeType,
                        ;
                        $($tokens)*
                    }
                }
            }
        };
        assert_expected_create_html_tokens(tokens, expected);
    }

    #[track_caller]
    fn assert_expected_create_html_tokens(start: TokenStream, expected: TokenStream) {
        let html: CreateHtmlMacro = syn::parse2(start).unwrap();
        assert_eq!(html.into_tokens().to_string(), expected.to_string());
    }
}
