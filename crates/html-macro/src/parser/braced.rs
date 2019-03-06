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

        let node_idx = &mut self.current_node_idx;
        let parent_to_children = &mut self.parent_to_children;
        let parent_stack = &mut self.parent_stack;
        let tokens = &mut self.tokens;
        let node_order = &mut self.node_order;

        let open_tag_end = match self.recent_span_locations.most_recent_open_tag_end.as_ref() {
            Some(open_tag_end) => Some((open_tag_end.line, open_tag_end.column)),
            None => None,
        };

        // We ignore this check if the last tag kind was text since the text
        // parser will already handle inserting spaces before and after text.
        let last_tag_kind_was_text = self.last_tag_kind == Some(TagKind::Text);

        // We ignore this check if the last tag kind was brace since the previous brace
        // parser will already handle inserting spaces before this current brace if needed.
        let last_tag_kind_was_brace = self.last_tag_kind == Some(TagKind::Braced);

        // TODO: Only allow one statement per block. Put a quote_spanned! compiler error if
        // there is more than 1 statement. Add a UI test for this.
        block.stmts.iter().enumerate().for_each(|(stmt_idx, stmt)| {
            if *node_idx == 0 {
                // Here we handle a block being the root node of an `html!` call
                //
                // html { { some_node }  }
                let node = quote! {
                    let node_0: VirtualNode = #stmt.into();
                };
                tokens.push(node);
            } else {
                // If this is the first node in the block we'll check to see if there is a space
                // between this block and the previous closing tag.
                // If so we'll insert a VirtualNode::text(" ") just in case the block contains
                // a text element. This way there will be space before the text.
                //
                // let some_var = "hello"
                // let another_var = "world";
                //
                // html! { <div>{some_var}</div> }  -> would not get a " " inserted
                //
                // html! { <div> {some_var}</div> } -> would get a " " inserted
                if stmt_idx == 0 && !last_tag_kind_was_text && !last_tag_kind_was_brace {
                    if let Some(open_tag_end) = open_tag_end {
                        if open_tag_end.0 != brace_span.start().line
                            || brace_span.start().column - open_tag_end.1 > 0
                        {
                            // FIXME BEFORE MERGE: Add a method to generate a new node that
                            // increments our node_idx. Pass a span to that method
                            let node_name = format!("node_{}", node_idx);
                            let node_name = Ident::new(node_name.as_str(), brace_span.clone());

                            let space = quote! {
                                let #node_name: IterableNodes = VirtualNode::text(" ").into();
                            };
                            tokens.push(space);

                            // TODO BEFORE MERGE: Exact same code repeated below. Normalize

                            let parent_idx = *&parent_stack[parent_stack.len() - 1].0;

                            parent_to_children
                                .get_mut(&parent_idx)
                                .expect("Parent of this text node")
                                .push(*node_idx);
                            node_order.push(*node_idx);

                            *node_idx += 1;
                        }
                    }
                }

                // Here we handle a block being a descendant within some html! call.
                //
                // The descendant should implement Into<IterableNodes>
                //
                // html { <div> { some_node } </div> }

                let node_name = format!("node_{}", node_idx);
                let node_name = Ident::new(node_name.as_str(), stmt.span());

                let nodes = quote! {
                    let #node_name: IterableNodes = #stmt.into();
                };
                tokens.push(nodes);

                // TODO BEFORE MERGE: Exact same code repeated above. Normalize

                let parent_idx = *&parent_stack[parent_stack.len() - 1].0;

                parent_to_children
                    .get_mut(&parent_idx)
                    .expect("Parent of this text node")
                    .push(*node_idx);
                node_order.push(*node_idx);

                *node_idx += 1;

                if should_insert_space_after {
                    // FIXME BEFORE MERGE: Add a method to generate a new node that
                    // increments our node_idx. Pass a span to that method
                    let node_name = format!("node_{}", node_idx);
                    let node_name = Ident::new(node_name.as_str(), brace_span.clone());

                    let space = quote! {
                        let #node_name: IterableNodes = VirtualNode::text(" ").into();
                    };
                    tokens.push(space);

                    // TODO BEFORE MERGE: Exact same code repeated below. Normalize

                    let parent_idx = *&parent_stack[parent_stack.len() - 1].0;

                    parent_to_children
                        .get_mut(&parent_idx)
                        .expect("Parent of this text node")
                        .push(*node_idx);
                    node_order.push(*node_idx);

                    *node_idx += 1;
                }
            }
        });

        self.set_most_recent_block_start(brace_span);
    }
}
