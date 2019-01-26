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

#[derive(Debug)]
pub enum Tag {
    /// <div id="app" class=*CSS>
    Open { name: Ident, attrs: Vec<Attr> },
    /// </div>
    Close { name: Ident },
    /// html! { <div> Hello World </div> }
    ///
    ///  -> Hello world
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

/// id="my-id"
/// class="some classes"
/// etc...
#[derive(Debug)]
pub struct Attr {
    pub key: Ident,
    pub value: Expr,
}

impl Parse for Tag {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut input = input;

        // If it starts with a `<` it's either an open or close tag.
        //   ex: <div>
        //   ex: </em>
        if input.peek(Token![<]) {
            input.parse::<Token![<]>()?;

            let optional_close: Option<Token![/]> = input.parse()?;
            let is_open_tag = optional_close.is_none();

            if is_open_tag {
                return parse_open_tag(&mut input);
            } else {
                return parse_close_tag(&mut input);
            }
        }

        // { node_inside_block }
        if input.peek(Brace) {
            return parse_block(&mut input);
        }

        return parse_text_node(&mut input);
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

fn parse_block(input: &mut ParseStream) -> Result<Tag> {
    let content;
    let brace_token = braced!(content in input);

    let block_expr = content.call(Block::parse_within)?;

    let block = Box::new(Block {
        brace_token,
        stmts: block_expr,
    });

    Ok(Tag::Braced { block })
}

fn parse_text_node(input: &mut ParseStream) -> Result<Tag> {
    // Continue parsing tokens until we see a closing tag <
    let mut text_tokens = TokenStream::new();

    let mut text = "".to_string();

    let mut idx = 0;

    loop {
        if input.is_empty() {
            break;
        }

        let is_comma = input.peek(Token![,]);

        // TODO: If we peek a token that we aren't confident that we can choose the correct
        // spacing for almost all of the time just print a compiler error telling the user
        // to use the text macro instead of text tokens...
        //  { text!("My text") }
        if input.peek(Token![,]) {
            let _: TokenTree = input.parse()?;
            text += ",";
        } else if input.peek(Token![!]) {
            let _: TokenTree = input.parse()?;
            text += "!";
        } else if input.peek(Token![.]) {
            let _: TokenTree = input.parse()?;
            text += ".";
        } else {
            let tt: TokenTree = input.parse()?;

            if idx != 0 {
                text += " ";
            }

            text += &tt.to_string();
        }

        let peek_closing_tag = input.peek(Token![<]);
        let peek_start_block = input.peek(Brace);

        if peek_closing_tag || peek_start_block {
            break;
        }

        idx += 1;
    }

    Ok(Tag::Text { text })
}
