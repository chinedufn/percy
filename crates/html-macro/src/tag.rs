use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::token::Brace;
use syn::{braced, Block, Expr, Ident, Token};

/// The different kinds of tokens that we parse.
///
/// TODO: A better name than tag since not all of these are tags
#[derive(Debug)]
pub enum Tag {
    /// <div id="app" class=*CSS>
    /// <br />
    Open {
        name: Ident,
        attrs: Vec<Attr>,
        open_bracket_span: Span,
        closing_bracket_span: Span,
        is_self_closing: bool,
    },
    /// </div>
    Close {
        name: Ident,
        first_angle_bracket_span: Span,
    },
    /// html! { <div> Hello World </div> }
    ///
    ///  -> Hello world
    ///
    /// start_span -> the span for the first token within the text
    /// end_span -> the span for the last token within the text
    Text {
        text: String,
        start_span: Option<Span>,
        end_span: Option<Span>,
    },
    /// let text_var = VirtualNode::text("3");
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
    Braced { block: Box<Block>, brace_span: Span },
}

/// The different kinds of tokens that we parse.
///
/// TODO: A better name than tag since not all of these are tags
#[derive(Debug, Eq, PartialEq)]
pub enum TagKind {
    Open,
    Close,
    Text,
    Braced,
}

/// id="my-id"
/// class="some classes"
/// etc...
#[derive(Debug)]
pub struct Attr {
    key: TokenStream,
    key_span: Span,
    value: Expr,
}
impl Attr {
    /// Get the attribute's key as a String.
    ///
    /// If the attribute's tokens were `quote!{ http - equiv }`
    /// the string will be "http-equiv".
    pub fn key_string(&self) -> String {
        self.key.to_string().replace(" ", "")
    }

    /// Get the span for the attribute's key.
    pub fn key_span(&self) -> Span {
        self.key_span
    }

    /// Get the attribute's value, such as the "refresh" in `<div http-equiv = "refresh"> </div>.
    pub fn value(&self) -> &Expr {
        &self.value
    }
}

impl Parse for Tag {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut input = input;

        // If it starts with a `<` it's either an open or close tag.
        //   ex: <div>
        //   ex: </em>
        if input.peek(Token![<]) {
            let first_angle_bracket_span = input.parse::<Token![<]>()?;
            let first_angle_bracket_span = first_angle_bracket_span.span();

            let optional_close: Option<Token![/]> = input.parse()?;
            let is_open_tag = optional_close.is_none();

            if is_open_tag {
                return parse_open_tag(&mut input, first_angle_bracket_span);
            } else {
                return parse_close_tag(&mut input, first_angle_bracket_span);
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
fn parse_open_tag(input: &mut ParseStream, open_bracket_span: Span) -> Result<Tag> {
    let name: Ident = input.parse()?;

    let attrs = parse_attributes(input)?;

    let is_self_closing: Option<Token![/]> = input.parse()?;
    let is_self_closing = is_self_closing.is_some();

    let closing_bracket = input.parse::<Token![>]>()?;
    let closing_bracket_span = closing_bracket.span();

    Ok(Tag::Open {
        name,
        attrs,
        open_bracket_span,
        closing_bracket_span,
        is_self_closing,
    })
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
    while input.peek(Ident)
        || input.peek(Token![as])
        || input.peek(Token![async])
        || input.peek(Token![for])
        || input.peek(Token![loop])
        || input.peek(Token![type])
    {
        let (key, key_span) = parse_attribute_key(input)?;

        println!("PARSING EQUALS");
        // =
        input.parse::<Token![=]>()?;
        println!("PARSED EQUALS");

        // Continue parsing tokens until we see the next attribute or a closing > tag
        let mut value_tokens = TokenStream::new();

        loop {
            let tt: TokenTree = input.parse()?;
            value_tokens.extend(Some(tt));

            let next_token_is_attrib_key = input.peek(Ident)
                || input.peek(Token![as])
                || input.peek(Token![async])
                || input.peek(Token![for])
                || input.peek(Token![loop])
                || input.peek(Token![type]);

            // The `=` in:
            //   class="hello-world"
            //
            // Or the `-` in:
            //   http-equiv="refresh"
            let next_next_token_is_equals_or_hyphen =
                input.peek2(Token![=]) || input.peek2(Token![-]);

            let peek_start_of_next_attr =
                next_token_is_attrib_key && next_next_token_is_equals_or_hyphen;

            let peek_end_of_tag = input.peek(Token![>]);

            let peek_self_closing = input.peek(Token![/]);

            if peek_end_of_tag || peek_start_of_next_attr || peek_self_closing {
                break;
            }
        }

        let value: Expr = syn::parse2(value_tokens)?;

        attrs.push(Attr {
            key,
            key_span,
            value,
        });
    }

    Ok(attrs)
}

/// Parse an attribute key such as the "http-equiv" in
/// `<meta http-equiv="refresh" />`
fn parse_attribute_key(input: &mut ParseStream) -> Result<(TokenStream, Span)> {
    let first_key_segment = parse_attribute_key_segment(input)?;

    let maybe_hyphen: Option<Token![-]> = input.parse()?;

    let attribute_key;
    if let Some(hyphen) = maybe_hyphen {
        let next_segment = parse_attribute_key_segment(input)?;

        let combined_span = first_key_segment
            .span()
            .join(hyphen.span())
            .unwrap()
            .join(next_segment.span())
            .unwrap();

        attribute_key = (
            quote_spanned! {combined_span=> #first_key_segment - #next_segment },
            combined_span,
        );
    } else {
        attribute_key = (
            first_key_segment.to_token_stream(),
            first_key_segment.span(),
        );
    }

    Ok(attribute_key)
}

/// Parse a segment within an attribute key, such as the "http" or "equiv" in
/// `<meta http-equiv="refresh" />`
fn parse_attribute_key_segment(input: &mut ParseStream) -> Result<Ident> {
    // <link rel="stylesheet" type="text/css"
    //   .. as, async, for, loop, type need to be handled specially since they are keywords
    let maybe_as_key: Option<Token![as]> = input.parse()?;
    let maybe_async_key: Option<Token![async]> = input.parse()?;
    let maybe_for_key: Option<Token![for]> = input.parse()?;
    let maybe_loop_key: Option<Token![loop]> = input.parse()?;
    let maybe_type_key: Option<Token![type]> = input.parse()?;

    let key = if let Some(as_key) = maybe_as_key {
        Ident::new("as", as_key.span())
    } else if let Some(async_key) = maybe_async_key {
        Ident::new("async", async_key.span())
    } else if let Some(for_key) = maybe_for_key {
        Ident::new("for", for_key.span())
    } else if let Some(loop_key) = maybe_loop_key {
        Ident::new("loop", loop_key.span())
    } else if let Some(type_key) = maybe_type_key {
        Ident::new("type", type_key.span())
    } else {
        input.parse()?
    };
    Ok(key)
}

/// </div>
fn parse_close_tag(input: &mut ParseStream, first_angle_bracket_span: Span) -> Result<Tag> {
    let name: Ident = input.parse()?;

    input.parse::<Token![>]>()?;

    Ok(Tag::Close {
        name,
        first_angle_bracket_span,
    })
}

fn parse_block(input: &mut ParseStream) -> Result<Tag> {
    let content;
    let brace_token = braced!(content in input);

    let brace_span = brace_token.span.open();

    let block_expr = content.call(Block::parse_within)?;

    let block = Box::new(Block {
        brace_token,
        stmts: block_expr,
    });

    Ok(Tag::Braced { block, brace_span })
}

/// Parse a sequence of tokens until we run into a closing tag
///   html! { <div> Hello World </div> }
/// or a brace
///   html! { <div> Hello World { Braced } </div>
///
/// So, in the second case, there would be two VText nodes created. "Hello World" and "Braced".
///
/// Later in parser/text.rs we'll look at how close the VText nodes are to their neighboring tags
/// to determine whether or not to insert spacing.
///
/// So, in the examples above, since the opening "<div>" has a space after it we'll later transform
/// "Hello World" into " Hello World" in parser/tag.rs
fn parse_text_node(input: &mut ParseStream) -> Result<Tag> {
    // Continue parsing tokens until we see a closing tag <
    let _text_tokens = TokenStream::new();

    let mut text = "".to_string();

    let mut idx = 0;

    let mut start_span = None;

    let mut most_recent_span: Option<Span> = None;

    loop {
        if input.is_empty() {
            break;
        }

        let tt: TokenTree = input.parse()?;

        if idx == 0 {
            start_span = Some(tt.span());
            most_recent_span = Some(tt.span());
        }

        // TODO: Properly handle whitespace and new lines
        // https://github.com/chinedufn/percy/pull/97#discussion_r263039215
        if idx != 0 {
            if let Some(most_recent_span) = most_recent_span {
                let current_span_start = tt.span().start();
                let most_recent_span_end = most_recent_span.end();

                let spans_on_different_lines = current_span_start.line != most_recent_span_end.line;

                // Contraptions such as "Aren't" give the "'" and the "t" the
                // same span, even though they get parsed separately when calling
                // input.parse::<TokenTree>().
                // As in - it takes two input.parse calls to get the "'" and "t",
                // even though they have the same span.
                // This might be a bug in syn - but regardless we address this by
                // not inserting a space in this case.
                let span_comes_before_previous_span = current_span_start.column
                    < most_recent_span_end.column
                    && !spans_on_different_lines;

                // Spans are on different lines, insert space
                if spans_on_different_lines {
                    text += " ";
                } else if !span_comes_before_previous_span
                    && current_span_start.column - most_recent_span_end.column > 0
                {
                    text += " ";
                }
            }
        }

        text += &tt.to_string();

        most_recent_span = Some(tt.span());

        let peek_closing_tag = input.peek(Token![<]);
        let peek_start_block = input.peek(Brace);

        if peek_closing_tag || peek_start_block {
            break;
        }

        idx += 1;
    }

    Ok(Tag::Text {
        text,
        start_span,
        end_span: most_recent_span,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use std::collections::HashMap;
    use syn::Lit;

    /// Verify that we can parse opening tags.
    ///
    /// ## Testing Approach
    /// - Iterate over macro tokens
    ///   - Assert that the `Tag::Open` has the expected name, such as "div".
    ///   - Assert that the `Tag::Open` is detected as self-closing, such as with "<br />".
    ///   - Assert that the `Tag::Open` has the expected attributes, such as "<div id="hello">.
    #[test]
    fn open_tag_tests() {
        let tests = [
            // Verify that we can parse an open tag that does not have any attributes.
            (
                quote! { <div> },
                ExpectedTag {
                    name: "div",
                    attributes: vec![],
                    is_self_closing: false,
                },
            ),
            // Verify that we can parse a self-closing open tag that does not have any attributes.
            (
                quote! { <br /> },
                ExpectedTag {
                    name: "br",
                    attributes: vec![],
                    is_self_closing: true,
                },
            ),
            // Verify that we can parse an open tag that has one attribute.
            (
                quote! { <div id="hello"> },
                ExpectedTag {
                    name: "div",
                    attributes: vec![("id", "hello")],
                    is_self_closing: false,
                },
            ),
            // Verify that we can parse an open tag that has one hyphenated attribute.
            (
                quote! { <meta http-equiv="refresh" /> },
                ExpectedTag {
                    name: "meta",
                    attributes: vec![("http-equiv", "refresh")],
                    is_self_closing: true,
                },
            ),
            // Verify that we can parse an element that has a hyphenated attribute as its second
            // attribute.
            (
                quote! {
                  <path
                    d="M1,5 a2,2"
                    stroke-linejoin="miter"
                  />
                },
                ExpectedTag {
                    name: "path",
                    attributes: vec![("d", "M1,5 a2,2"), ("stroke-linejoin", "miter")],
                    is_self_closing: true,
                },
            ),
        ];

        for (tokens, expected_tag) in tests {
            let tokens_string = tokens.to_string();
            let tag: Tag = syn::parse2(tokens).unwrap();

            match tag {
                Tag::Open {
                    name,
                    attrs,
                    is_self_closing,
                    ..
                } => {
                    assert_eq!(&name.to_string(), expected_tag.name, "{}", tokens_string);

                    assert_eq!(
                        is_self_closing, expected_tag.is_self_closing,
                        "{}",
                        tokens_string
                    );

                    let expected_attrs: HashMap<&'static str, &'static str> =
                        expected_tag.attributes.into_iter().collect();

                    assert_eq!(attrs.len(), expected_attrs.len(), "{}", tokens_string);
                    for attr in attrs {
                        let attr_key = attr.key_string();

                        let Expr::Lit(attr_val) = attr.value else {
                            panic!()
                        };
                        let Lit::Str(attr_val_str) = attr_val.lit else {
                            panic!()
                        };

                        let expected_val = expected_attrs
                            .get(attr_key.as_str())
                            .map(|val| val.to_string());

                        assert_eq!(Some(attr_val_str.value()), expected_val,);
                    }
                }
                not_open => panic!("Should have been an open tag. {:?}", not_open),
            }
        }
    }

    struct ExpectedTag {
        name: &'static str,
        attributes: Vec<(&'static str, &'static str)>,
        is_self_closing: bool,
    }
}
