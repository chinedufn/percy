use crate::parser::HtmlParser;
use proc_macro2::Ident;
use quote::quote;
use syn::spanned::Spanned;
use syn::Block;

impl HtmlParser {
    /// Parse an incoming Tag::Braced text node
    pub(crate) fn parse_braced(&mut self, block: Box<Block>) {
        let idx = &mut self.current_idx;
        let parent_to_children = &mut self.parent_to_children;
        let parent_stack = &mut self.parent_stack;
        let tokens = &mut self.tokens;
        let node_order = &mut self.node_order;

        block.stmts.iter().for_each(|stmt| {
            if *idx == 0 {
                // Here we handle a block being the root node of an `html!` call
                //
                // html { { some_node }  }
                let node = quote! {
                    let node_0: VirtualNode = #stmt.into();
                };
                tokens.push(node);
            } else {
                // Here we handle a block being a descendant within some html! call.
                //
                // The descendant should implement Into<IterableNodes>
                //
                // html { <div> { some_node } </div> }

                let node_name = format!("node_{}", idx);
                let node_name = Ident::new(node_name.as_str(), stmt.span());

                let nodes = quote! {
                    let #node_name: IterableNodes = #stmt.into();
                };
                tokens.push(nodes);

                let parent_idx = *&parent_stack[parent_stack.len() - 1].0;

                parent_to_children
                    .get_mut(&parent_idx)
                    .expect("Parent of this text node")
                    .push(*idx);
                node_order.push(*idx);

                *idx += 1;
            }
        })
    }
}
