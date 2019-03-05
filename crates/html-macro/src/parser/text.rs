use crate::parser::{is_self_closing, HtmlParser};
use crate::tag::{Attr, Tag};
use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned};
use syn::Expr;

impl HtmlParser {
    /// Parse an incoming Tag::Text text node
    pub(crate) fn parse_text(&mut self, text: String) {
        let idx = &mut self.current_idx;
        let parent_to_children = &mut self.parent_to_children;
        let parent_stack = &mut self.parent_stack;
        let tokens = &mut self.tokens;
        let node_order = &mut self.node_order;

        if *idx == 0 {
            node_order.push(0);
            // TODO: This is just a consequence of bad code. We're pushing this to make
            // things work but in reality a text node isn't a parent ever.
            // Just need to make the code DRY / refactor so that we can make things make
            // sense vs. just bolting things together.
            parent_stack.push((0, Ident::new("unused", Span::call_site())));
        }

        // TODO: Figure out how to use spans
        let var_name = Ident::new(format!("node_{}", idx).as_str(), Span::call_site());

        let text_node = quote! {
            let mut #var_name = VirtualNode::text(#text);
        };

        tokens.push(text_node);

        if *idx == 0 {
            *idx += 1;
            return;
        }

        let parent_idx = &parent_stack[parent_stack.len() - 1];

        node_order.push(*idx);

        parent_to_children
            .get_mut(&parent_idx.0)
            .expect("Parent of this text node")
            .push(*idx);

        *idx += 1;
    }
}
