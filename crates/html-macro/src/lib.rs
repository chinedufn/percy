extern crate proc_macro;

use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Expr, Ident, Token};

// FIXME: Play around and get things working but add thorough commenting
// once it's all put together

#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(input as Html);

    eprintln!("parsed = {:#?}", parsed);

    let mut tokens = vec![];

    for (idx, tag) in parsed.tags.into_iter().enumerate() {
        match tag {
            Tag::Open { name, attrs } => {
                // The root node is named `node_0`. All of it's descendants are node_1.. node_2.. etc.
                // This just comes from the `idx` variable
                // TODO: Not sure what the span is supposed to be so I just picked something..
                let var_name = Ident::new(format!("node_{}", idx).as_str(), name.span());

                let name = format!("{}", name);
                let node = quote! {
                    let mut #var_name = VirtualNode::new(#name);
                };
                tokens.push(node);

                for attr in attrs.iter() {
                    let key = format!("{}", attr.key);
                    let value = &attr.value;

                    let insert_attribute = quote! {
                       #var_name.props.insert(#key.to_string(), #value.to_string());
                    };

                    tokens.push(insert_attribute);
                }
            }
            Tag::Close { name } => {}
        };
    }

    let virtual_nodes = quote! {
        {
            #(#tokens)*
            // Root node is always named node_0
            node_0
        }
    };
    eprintln!("virtual_nodes  = {:#?}", virtual_nodes);

    virtual_nodes.into()
}

#[derive(Debug)]
struct Html {
    // TODO: also support content between the tags
    tags: Vec<Tag>,
}

#[derive(Debug)]
enum Tag {
    /// `<div id="app" class=*CSS>`
    Open { name: Ident, attrs: Vec<Attr> },
    /// `</div>`
    Close { name: Ident },
}

#[derive(Debug)]
struct Attr {
    key: Ident,
    value: Expr,
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

impl Parse for Tag {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![<]>()?;
        let optional_close: Option<Token![/]> = input.parse()?;
        let is_close_tag = optional_close.is_some();
        let name: Ident = input.parse()?;

        let mut attrs = Vec::new();
        while input.peek(Ident) && !is_close_tag {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            let mut value_tokens = TokenStream::new();
            loop {
                let tt: TokenTree = input.parse()?;
                value_tokens.extend(Some(tt));

                let peek_end_of_tag = input.peek(Token![>]);
                let peek_start_of_next_attr = input.peek(Ident) && input.peek2(Token![=]);
                if peek_end_of_tag || peek_start_of_next_attr {
                    break;
                }
            }

            let value: Expr = syn::parse2(value_tokens)?;
            attrs.push(Attr { key, value });
        }

        input.parse::<Token![>]>()?;

        if is_close_tag {
            Ok(Tag::Close { name })
        } else {
            Ok(Tag::Open { name, attrs })
        }
    }
}
