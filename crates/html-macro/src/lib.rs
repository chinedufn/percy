extern crate proc_macro;

use crate::config::{HtmlAndConfig, HtmlConfig};
use crate::create::CreateHtmlMacro;
use crate::parser::HtmlParser;
use crate::tag::Tag;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, parse_quote};

mod config;
mod create;

mod maybe_dollar;
mod parser;
mod tag;

/// Creates a custom `html!` macro with the given settings.
/// The generated `html!` macro calls [`html_with_config`] under the hood.
/// ```no_run
/// # use html_macro::define_html_macro;
/// # use percy_dom::{VirtualNode, VirtualElement, VirtualText, IterableNodes};
///
/// // This creates a macro called `my_html!`
/// define_html_macro! {
///     /// This documentation comment will get added to the generated `my_html!` macro.
///     my_html!
///     real_dom = (),
///
///     // Optionally configure the path to the underlying macro to call.
///     calls = html_macro::html_with_config,
/// };
///
/// // `my_html!` can now be used to create `VirtualNode`s.
/// let node: VirtualNode<()> = my_html! { <div>hello world</div> };
/// ```
#[proc_macro]
pub fn define_html_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(input as CreateHtmlMacro);
    parsed.into_tokens().into()
}

/// Build a `VirtualNode` from a token stream.
/// Calls [`html_with_config`] with the following configuration:
/// - Uses `web_sys::Window` as the `RealDom`.
///
/// ## Examples
// Ignored to avoid having to add `web_sys` as a dev-dependency.
/// ```ignore
/// # use html_macro::html;
/// let div = html! { <div> Welcome to the html! procedural macro! </div> };
/// ```
#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let html = parse_macro_input!(input as Html);

    HtmlAndConfig {
        html,
        config: HtmlConfig {
            real_dom_ty: parse_quote!(web_sys::Window),
        },
    }
    .into_tokens()
    .into()
}

/// Build a `VirtualNode` from a token stream.
///
/// Takes one or more comma-separated settings to configure, followed by a `;`, followed by by
/// HTML.
///
/// ## Examples
// Ignored to avoid having to add `web_sys` as a dev-dependency.
/// ```ignore
/// # use html_macro::html_with_config;
///
/// // Generates a `VirtualNode::<web_sys::Window>`
/// let node = html_with_config! {
///     real_dom = web_sys::Window;
///     <div> hello world </div>
/// };
///
/// ```
///
/// ```
/// # use html_macro::html_with_config;
/// # use percy_dom::{VirtualNode, VirtualElement, VirtualText, IterableNodes};
///
/// // Generates a `VirtualNode::<()>`
/// let node = html_with_config! {
///     real_dom = ();
///     <div> hello world </div>
/// };
/// ```
#[proc_macro]
pub fn html_with_config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(input as HtmlAndConfig);
    parsed.into_tokens().into()
}

/// ...
#[proc_macro]
pub fn reflect_tokens(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(input as proc_macro2::TokenStream);
    parsed.into()
}

#[derive(Debug)]
struct Html {
    tags: Vec<Tag>,
}

impl Parse for Html {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut tags = Vec::new();

        while !input.is_empty() {
            let tag: Tag = input.parse()?;
            tags.push(tag);
        }

        Ok(Html { tags })
    }
}

struct HtmlToTokensConfig {
    real_dom_ty: syn::Type,
}

impl Html {
    /// Start with parsed tags such as `<div hello="world"></div>`.
    /// End with Rust code such as `let node = VirtualNode::new_element("div"); ...`
    fn into_tokens(self, config: HtmlToTokensConfig) -> proc_macro2::TokenStream {
        let html = self;

        let mut html_parser = HtmlParser::new();

        let parsed_tags_len = html.tags.len();

        for (idx, tag) in html.tags.iter().enumerate() {
            let mut next_tag = None;

            if parsed_tags_len - 1 > idx {
                next_tag = Some(&html.tags[idx + 1])
            }

            html_parser.push_tag(tag, next_tag, &config.real_dom_ty);
        }

        html_parser.finish()
    }
}

#[cfg(test)]
pub(self) mod tests {
    //! This crate's tests assert that the generated tokens match what we expect.
    //!
    //! `crates/html-macro-test` contains tests that invoke the `html!` and confirm that the
    //! returned node tree has the expected properties.

    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;

    /// Verify that after invoking the `html!` macro, the compiler output matches what we expect.
    /// Primarily used to test compile-time error messages.
    #[test]
    fn ui() {
        let t = trybuild::TestCases::new();

        let ui_tests = concat!(env!("CARGO_MANIFEST_DIR"), "/ui_tests/*.rs");
        t.compile_fail(ui_tests);
    }

    /// Verify that we specify a virtual element's `RealDom` generic parameter.
    #[test]
    fn specifies_generic_param() {
        let tests = [
            // Root element
            (
                quote! { <div> </div> },
                quote! {
                    {
                        let mut node_0 = VirtualNode::<web_sys::Window>::new_element("div");
                        node_0
                    }
                },
            ),
            // Root text
            (
                quote! { hello },
                quote! {
                    {
                      let mut node_0 = VirtualNode::<web_sys::Window>::new_text("hello");
                      node_0
                    }
                },
            ),
            // Root block
            (
                quote! {
                    { some_variable }
                },
                quote! {
                    {
                        let node_0: VirtualNode::<web_sys::Window> = some_variable.into();
                        node_0
                    }
                },
            ),
            // Block inside element
            (
                quote! {
                    <div> { some_variable } </div>
                },
                quote! {
                    {
                        let mut node_0 = VirtualNode::<web_sys::Window>::new_element("div");
                        let mut node_1: IterableNodes<web_sys::Window> = (some_variable).into();
                        if let Some(ref mut element_node) = node_0.as_elem_mut() {
                            element_node.children.extend(node_1.into_iter());
                        } else {
                            // TODO: Change our codegen to create a `VirtualElement`, push the
                            //  children, then at the end create the `VirtualNode`.
                            //  This way we can remove this `unreachable!()` branch entirely.
                            { unreachable!("Non-elements cannot have children"); } ;
                        }
                        node_0
                    }
                },
            ),
        ];

        for (tokens, expected) in tests {
            assert_expected_html_tokens(tokens, expected);
        }
    }

    #[track_caller]
    pub(super) fn assert_expected_html_tokens(start: TokenStream, expected: TokenStream) {
        let html: Html = syn::parse2(start).unwrap();
        let dom_ty: syn::Type = syn::parse2(quote! { web_sys::Window}).unwrap();
        assert_eq!(
            html.into_tokens(HtmlToTokensConfig {
                real_dom_ty: dom_ty,
            })
            .to_string(),
            expected.to_string()
        );
    }
}
