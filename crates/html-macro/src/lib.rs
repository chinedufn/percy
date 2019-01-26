extern crate proc_macro;

use crate::parser::HtmlParser;
use crate::tag::{Attr, Tag};
use proc_macro2::Literal;
use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use std::collections::HashMap;
use syn::export::Span;
use syn::group::Group;
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::token::Brace;
use syn::{braced, parse_macro_input, Block, Expr, Ident, Token};

mod parser;
mod tag;

#[proc_macro]
pub fn text(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let text: Expr = syn::parse(input).expect("Text variable");
    let text = quote! {
        VirtualNode::text(#text)
    };

    text.into()
}

/// Used to generate VirtualNode's from a TokenStream.
///
/// html! { <div> Welcome to the html! procedural macro! </div> }
#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(input as Html);

    let mut html_parser = HtmlParser::new();

    for tag in parsed.tags.into_iter() {
        html_parser.push_tag(tag);
    }

    html_parser.finish().into()
}

#[derive(Debug)]
struct Html {
    tags: Vec<Tag>,
}

impl Parse for Html {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut tags = Vec::new();

        while !input.is_empty() {
            let tag: Tag = input.parse()?;
            tags.push(tag);
        }

        Ok(Html { tags })
    }
}

