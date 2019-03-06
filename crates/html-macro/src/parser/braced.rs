use crate::parser::HtmlParser;
use crate::tag::{Tag, TagKind};
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::spanned::Spanned;
use syn::Block;

impl HtmlParser {
    /// Parse an incoming Tag::Braced text node
    pub(crate) fn parse_braced(
        &mut self,
        block: &Box<Block>,
        brace_span: &Span,
        next_tag: Option<&Tag>,
    ) {
        // If
        //   1. The next tag is a closing bracket or another brace
        //   2. There is space between this brace and that next tag
        //
        // Then
        //   We'll insert some spacing after this brace.
        //
        // This ensures that we properly maintain spacing between two neighboring braced
        // text nodes
        //
        // html! { <div>{ This Brace } { Space WILL be inserted }</div>
        //   -> <div>This Brace Space WILL be inserted</div>
        //
        // html! { <div>{ This Brace }{ Space WILL NOT be inserted }</div>
        //   -> <div>This BraceSpace WILL NOT be inserted</div>
        let should_insert_space_after = match next_tag {
            Some(Tag::Close {
                first_angle_bracket_span,
                ..
            }) => self.separated_by_whitespace(brace_span, &first_angle_bracket_span),
            Some(Tag::Braced {
                brace_span: next_brace_span,
                ..
            }) => self.separated_by_whitespace(brace_span, &next_brace_span),
            _ => false,
        };

        // TODO: Only allow one statement per block. Put a quote_spanned! compiler error if
        // there is more than 1 statement. Add a UI test for this.
        block.stmts.iter().for_each(|stmt| {
            if self.current_node_idx == 0 {
                // Here we handle a block being the root node of an `html!` call
                //
                // html { { some_node }  }
                let node = quote! {
                    let node_0: VirtualNode = #stmt.into();
                };
                self.push_tokens(node);
            } else {
                // We'll check to see if there is a space between this block and the previous open
                // tag's closing brace.
                //
                // If so we'll insert a VirtualNode::text(" ") just in case the block contains
                // a text element. This way there will be space before the text.
                //
                // let some_var = "hello"
                // let another_var = "world";
                //
                // html! { <div>{some_var}</div> }  -> would not get a " " inserted
                //
                // html! { <div> {some_var}</div> } -> would get a " " inserted
                if let Some(open_tag_end) =
                    self.recent_span_locations.most_recent_open_tag_end.as_ref()
                {
                    if self.last_tag_kind == Some(TagKind::Open)
                        && self.separated_by_whitespace(open_tag_end, brace_span)
                    {
                        self.push_virtual_text_space_tokens(stmt.span());
                    }
                }

                // Here we handle a block being a descendant within some html! call.
                //
                // The descendant should implement Into<IterableNodes>
                //
                // html { <div> { some_node } </div> }
                self.push_iterable_nodes(stmt);

                if should_insert_space_after {
                    self.push_virtual_text_space_tokens(stmt.span())
                }
            }
        });

        self.set_most_recent_block_start(brace_span.clone());
    }
}
