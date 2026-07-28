use crate::maybe_dollar::TypeAndMaybeDollar;
use crate::{Html, HtmlToTokensConfig};
use syn::Token;
use syn::parse::{Parse, ParseStream};

pub(super) struct HtmlAndConfig {
    pub html: Html,
    pub config: HtmlConfig,
}

#[derive(Debug)]
pub(crate) struct HtmlConfig {
    pub real_dom_ty: TypeAndMaybeDollar,
}

#[derive(Default)]
struct ParsedConfig {
    real_dom: Option<TypeAndMaybeDollar>,
    root_node_ident_override: Option<syn::Ident>,
}

pub(crate) enum ParsedHtmlConfigField {
    RealDom(TypeAndMaybeDollar),
    /// Used for macro hygiene.
    /// See [`crate::HtmlParser`]'s documentation for more information.
    RootNode(syn::Ident),
}

impl Parse for HtmlAndConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let config = HtmlConfig::parse(input)?;
        input.parse::<Token![;]>()?;
        let html = input.parse::<Html>()?;

        Ok(Self { html, config })
    }
}

impl Parse for HtmlConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut fields = Vec::new();

        loop {
            let next_is_semicolon = input.peek(Token![;]);
            let has_ended = input.is_empty();

            if next_is_semicolon {
                break;
            } else if has_ended {
                break;
            }

            let field = input.parse::<ParsedHtmlConfigField>()?;
            fields.push(field);

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(HtmlConfig::from_fields(fields))
    }
}

impl HtmlConfig {
    pub(crate) fn from_fields(fields: Vec<ParsedHtmlConfigField>) -> Self {
        let mut parsed_config = ParsedConfig::default();

        for field in fields {
            match field {
                ParsedHtmlConfigField::RealDom(real_dom_ty) => {
                    // TODO: Add UI test that we error when the field is already set.
                    //  As in, `real-dom-ty` appears twice in the `html_with_config!` macro
                    parsed_config.real_dom = Some(real_dom_ty);
                }
                ParsedHtmlConfigField::RootNode(root_node) => {
                    parsed_config.root_node_ident_override = Some(root_node);
                }
            }
        }

        HtmlConfig {
            // TODO: Add UI test that we error when `real_dom` is `None`
            real_dom_ty: parsed_config.real_dom.unwrap(),
        }
    }
}

impl Parse for ParsedHtmlConfigField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse::<syn::Ident>()?;
        let _equals = input.parse::<Token![=]>()?;

        let field = match key.to_string().as_str() {
            "real_dom" => {
                let ty = input.parse::<TypeAndMaybeDollar>()?;
                ParsedHtmlConfigField::RealDom(ty)
            }
            "root_node_ident" => {
                let ident = input.parse::<syn::Ident>()?;
                ParsedHtmlConfigField::RootNode(ident)
            }
            _ => todo!("add UI test that we error when there is an unrecognized key"),
        };
        Ok(field)
    }
}

impl HtmlAndConfig {
    pub fn into_tokens(self) -> proc_macro2::TokenStream {
        self.html.into_tokens(HtmlToTokensConfig {
            real_dom_ty: self.config.real_dom_ty.ty,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;

    /// Verify that we can use [`html_with_config`] to configure the `RealDom`.
    #[test]
    fn configure_real_dom() {
        let tokens = quote! {
            real_dom = SomeType ;
            <div></div>
        };
        let expected = quote! {
            {
                let mut node_0 = VirtualNode::<SomeType>::new_element("div");
                node_0
            }
        };
        assert_expected_html_with_config_tokens(tokens, expected);
    }

    /// Verify that we can have a trailing comma after the last field.
    #[test]
    fn supports_trailing_comma() {
        let tokens = quote! {
            real_dom = SomeType, ;
            <div></div>
        };
        let expected = quote! {
            {
                let mut node_0 = VirtualNode::<SomeType>::new_element("div");
                node_0
            }
        };
        assert_expected_html_with_config_tokens(tokens, expected);
    }

    #[track_caller]
    fn assert_expected_html_with_config_tokens(start: TokenStream, expected: TokenStream) {
        let html: HtmlAndConfig = syn::parse2(start).unwrap();
        assert_eq!(html.into_tokens().to_string(), expected.to_string());
    }
}
