extern crate proc_macro;

use proc_macro2::Literal;
use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use std::collections::HashMap;
use syn::export::Span;
use syn::group::Group;
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{braced, parse_macro_input, Block, Expr, Ident, Token};

// TODO: BREADCRUMB - look at the diff on GitHub and clean the code up.
// Replace text node from var creation with text! macro ..
// Update book with a big block of HTML illustrating everything. Save with README

// FIXME: Move to module
#[proc_macro]
pub fn text(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let text: Expr = syn::parse(input).expect("Text variable");
    let text = quote! {
        VirtualNode::text(#text)
    };

    text.into()
}

#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(input as Html);

    let mut tokens = vec![];

    // TODO: Manage node_order and parent_stack together so that we don't forget to change
    // one but not the other..
    let mut node_order = vec![];
    let mut parent_stack = vec![];

    let mut parent_children: HashMap<usize, Vec<usize>> = HashMap::new();
    parent_children.insert(0, vec![]);

    // TODO: A struct that manages all of these indices as we parse. Handle this when we refactor.
    // Before merging this all..
    let mut idx = 0;

    for tag in parsed.tags.into_iter() {
        match tag {
            Tag::Open { name, attrs } => {
                if idx == 0 {
                    node_order.push(0);
                    parent_stack.push(0);
                }

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
                    match value {
                        Expr::Closure(closure) => {
                            // TODO: Use this to decide Box<FnMut(_, _, _, ...)
                            let arg_count = closure.inputs.len();

                            let add_closure = quote! {
                                #[cfg(target_arch = "wasm32")]
                                {
                                  let closure = wasm_bindgen::prelude::Closure::wrap(
                                      Box::new(#value) as Box<FnMut(_)>
                                  );
                                  let closure = Box::new(closure);
                                  #var_name.events.0.insert(#key.to_string(), closure);
                                }
                            };

                            tokens.push(add_closure);
                        }
                        _ => {
                            let insert_attribute = quote! {
                                #var_name.props.insert(#key.to_string(), #value.to_string());
                            };
                            tokens.push(insert_attribute);
                        }
                    };
                }

                // The first open tag that we see is our root node so we won't worry about
                // giving it a parent
                if idx == 0 {
                    idx += 1;
                    continue;
                }

                let parent_idx = parent_stack[parent_stack.len() - 1];

                parent_stack.push(idx);
                node_order.push(idx);

                parent_children
                    .get_mut(&parent_idx)
                    .expect("Parent of this node")
                    .push(idx);

                parent_children.insert(idx, vec![]);

                idx += 1;
            }
            Tag::Close { name } => {
                parent_stack.pop();
            }
            Tag::Text { text } => {
                if idx == 0 {
                    node_order.push(0);
                    parent_stack.push(0);
                }

                // TODO: Figure out how to use spans
                let var_name = Ident::new(format!("node_{}", idx).as_str(), Span::call_site());

                let text_node = quote! {
                    let mut #var_name = VirtualNode::text(#text);
                };

                tokens.push(text_node);

                if idx == 0 {
                    idx += 1;
                    continue;
                }

                let parent_idx = parent_stack[parent_stack.len() - 1];

                node_order.push(idx);

                parent_children
                    .get_mut(&parent_idx)
                    .expect("Parent of this text node")
                    .push(idx);

                idx += 1;
            }
            Tag::Braced { block } => block.stmts.iter().for_each(|stmt| {
                if idx == 0 {
                    let node = quote! {
                        let node_0 = #stmt;
                    };
                    tokens.push(node);
                } else {
                    let node_name = format!("node_{}", idx);
                    let node_name = Ident::new(node_name.as_str(), stmt.span());

                    let nodes = quote! {
                        let #node_name = #stmt;
                    };
                    tokens.push(nodes);

                    let parent_idx = parent_stack[parent_stack.len() - 1];

                    parent_children
                        .get_mut(&parent_idx)
                        .expect("Parent of this text node")
                        .push(idx);
                    node_order.push(idx);

                    idx += 1;
                }
            }),
        };
    }

    if node_order.len() > 1 {
        for _ in 0..(node_order.len()) {
            let parent_idx = node_order.pop().unwrap();

            // TODO: Figure out how to really use spans
            let parent_name =
                Ident::new(format!("node_{}", parent_idx).as_str(), Span::call_site());

            let parent_children_indices = match parent_children.get(&parent_idx) {
                Some(children) => children,
                None => continue,
            };

            if parent_children_indices.len() > 0 {
                let create_children_vec = quote! {
                    #parent_name.children = Some(vec![]);
                };

                tokens.push(create_children_vec);

                for child_idx in parent_children_indices.iter() {
                    let children =
                        Ident::new(format!("node_{}", child_idx).as_str(), Span::call_site());

                    // TODO: Multiple .as_mut().unwrap() of children. Let's just do this once.
                    let push_children = quote! {
                        for child in #children.into_iter() {
                            #parent_name.children.as_mut().unwrap().push(child);
                        }
                    };
                    tokens.push(push_children);
                }
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

    virtual_nodes.into()
}

#[derive(Debug)]
struct Html {
    // TODO: also support content between the tags
    tags: Vec<Tag>,
}

#[derive(Debug)]
enum Tag {
    /// <div id="app" class=*CSS>
    Open { name: Ident, attrs: Vec<Attr> },
    /// </div>
    Close { name: Ident },
    /// html! { <div> Hello World </div> }
    ///
    ///  -> "Hello world"
    Text { text: String },
    /// let text_var = VirtualNode::from("3");
    ///
    /// let iter_nodes =
    ///   vec![
    ///     html!{ <div></div> },
    ///     html! {<span> </span>}
    ///   ];
    ///
    /// html! {
    ///   <div>
    ///     Here are some examples of blocks
    ///     { text_var }
    ///     { iter_nodes }
    ///     { html! { <div> </div> }
    ///   </div>
    /// }
    Braced { block: Box<Block> },
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
        let mut input = input;

        let content;

        // If it doesn't start with a < it's weird a text node or an expression
        let is_text_or_block = !input.peek(Token![<]);

        if is_text_or_block {
            // TODO: Move into parse_brace
            if !input.peek(Ident) && !input.peek(syn::Lit) {
                let brace_token = braced!(content in input);

                let block_expr = content.call(Block::parse_within)?;

                let block = Box::new(Block {
                    brace_token,
                    stmts: block_expr,
                });

                return Ok(Tag::Braced { block });
            }

            return parse_text_node(&mut input);
        }

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

fn parse_text_node(input: &mut ParseStream) -> Result<Tag> {
    // Continue parsing tokens until we see a closing tag <
    let mut text_tokens = TokenStream::new();

    let text = "".to_string();

    loop {
        if input.is_empty() {
            break;
        }

        let puncutation = input.peek(Token![,]);

        let tt: TokenTree = input.parse()?;
        //        eprintln!("tt.to_string() = {:#?}", tt.to_string());
        text_tokens.extend(Some(tt));

        let peek_closing_tag = input.peek(Token![<]);
        let peek_start_block = input.peek(syn::token::Brace);

        if peek_closing_tag || peek_start_block {
            break;
        }
    }

    let text = format!("{}", text_tokens);
    eprintln!("text = {:#?}", text);

    Ok(Tag::Text { text })
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
    while input.peek(Ident) || input.peek(Token![type]) {
        // <link rel="stylesheet" type="text/css"
        //   .. type needs to be handled specially since it's a keyword
        let maybe_type_key: Option<Token![type]> = input.parse()?;

        let key = if maybe_type_key.is_some() {
            Ident::new("type", maybe_type_key.unwrap().span())
        } else {
            input.parse()?
        };

        // =
        input.parse::<Token![=]>()?;

        // Continue parsing tokens until we see the next attribute or a closing > tag
        let mut value_tokens = TokenStream::new();

        loop {
            let tt: TokenTree = input.parse()?;
            value_tokens.extend(Some(tt));

            let has_attrib_key = input.peek(Ident) || input.peek(Token![type]);
            let peek_start_of_next_attr = has_attrib_key && input.peek2(Token![=]);

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
