extern crate proc_macro;

use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use std::collections::HashMap;
use syn::export::Span;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Expr, Ident, Token};

// FIXME: Play around and get things working but add thorough commenting
// once it's all put together

#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(input as Html);

    eprintln!("parsed = {:#?}", parsed);

    let mut tokens = vec![];

    // TODO: Manage node_order and parent_stack together so that we don't forget to change
    // one but not the other..
    let mut node_order = vec![0];
    let mut parent_stack = vec![0];

    let mut parent_children: HashMap<usize, Vec<usize>> = HashMap::new();
    parent_children.insert(0, vec![]);

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

                // The first open tag that we see is our root node so we won't worry about
                // giving it a parent
                if idx == 0 {
                    continue;
                }

                let parent_idx = parent_stack[parent_stack.len() - 1];

                parent_stack.push(idx);
                node_order.push(idx);

                parent_children
                    .get_mut(&parent_idx)
                    .expect("Parent")
                    .push(idx);

                parent_children.insert(idx, vec![]);
            }
            Tag::Close { name } => {
                parent_stack.pop();
            }
            Tag::Text { text } => {}
        };
    }

    for _ in 0..(node_order.len()) {
        let parent_idx = node_order.pop().unwrap();

        // TODO: Figure out how to really use spans
        let parent_name = Ident::new(format!("node_{}", parent_idx).as_str(), Span::call_site());

        let parent_children_indices = parent_children.get(&parent_idx).expect("Parent");

        if parent_children_indices.len() > 0 {
            let create_children_vec = quote! {
                #parent_name.children = Some(vec![]);
            };

            tokens.push(create_children_vec);

            for child_idx in parent_children_indices.iter() {
                let child_name =
                    Ident::new(format!("node_{}", child_idx).as_str(), Span::call_site());

                // TODO: Multiple .as_mut().unwrap() of children. Let's just do this once.
                let push_child = quote! {
                    #parent_name.children.as_mut().unwrap().push(#child_name);
                };
                tokens.push(push_child);
            }
        }
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
    /// "Hello world"
    Text { text: Expr },
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

// TODO: BREADCRUMB - Start by commenting this out so that I understand it.
// Then add support for text nodes
impl Parse for Tag {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut input = input;

        input.parse::<Token![<]>()?;

        let optional_close: Option<Token![/]> = input.parse()?;
        let is_open_tag = optional_close.is_none();

        if is_open_tag {
            parse_open_tag(&mut input)
        } else {
            parse_close_tag(&mut input)
        }
    }
}

/// `<div id="app" class=*CSS>`
fn parse_open_tag(input: &mut ParseStream) -> Result<Tag> {
    let name: Ident = input.parse()?;

    let attrs = parse_attributes(input)?;

    input.parse::<Token![>]>()?;

    Ok(Tag::Open { name, attrs })
}

/// Parse the attributes starting from something like:
///     id="app" class=*CSS>
///
/// As soon as we see
///     >
/// We know that the element has no more attributes and our loop will end
fn parse_attributes(input: &mut ParseStream) -> Result<Vec<Attr>> {
    let mut attrs = Vec::new();

    // Do we see an identifier such as `id`? If so proceed
    while input.peek(Ident) {
        // id
        let key: Ident = input.parse()?;

        // =
        input.parse::<Token![=]>()?;

        // Continue parsing tokens until we see the next attribute or a closing > tag
        let mut value_tokens = TokenStream::new();

        loop {
            let tt: TokenTree = input.parse()?;
            value_tokens.extend(Some(tt));

            let peek_start_of_next_attr = input.peek(Ident) && input.peek2(Token![=]);

            let peek_end_of_tag = input.peek(Token![>]);

            if peek_end_of_tag || peek_start_of_next_attr {
                break;
            }
        }

        let value: Expr = syn::parse2(value_tokens)?;

        attrs.push(Attr { key, value });
    }

    Ok(attrs)
}

/// </div>
fn parse_close_tag(input: &mut ParseStream) -> Result<Tag> {
    let name: Ident = input.parse()?;

    input.parse::<Token![>]>()?;

    Ok(Tag::Close { name })
}
