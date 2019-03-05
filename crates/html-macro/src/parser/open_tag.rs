use crate::parser::{is_self_closing, HtmlParser};
use crate::tag::Attr;
use proc_macro2::Ident;
use quote::quote;
use syn::Expr;

impl HtmlParser {
    /// Parse an incoming Tag::Open
    pub(crate) fn parse_open_tag(&mut self, name: Ident, attrs: Vec<Attr>) {
        let idx = &mut self.current_idx;
        let parent_to_children = &mut self.parent_to_children;
        let parent_stack = &mut self.parent_stack;
        let tokens = &mut self.tokens;
        let node_order = &mut self.node_order;

        // The root node is named `node_0`. All of it's descendants are node_1.. node_2.. etc.
        // This just comes from the `idx` variable
        // TODO: Not sure what the span is supposed to be so I just picked something..
        let var_name_node = Ident::new(format!("node_{}", idx).as_str(), name.span());
        let html_tag = format!("{}", name);

        let node = quote! {
            let mut #var_name_node = VirtualNode::element(#html_tag);
        };
        tokens.push(node);

        for attr in attrs.iter() {
            let key = format!("{}", attr.key);
            let value = &attr.value;
            match value {
                Expr::Closure(closure) => {
                    // TODO: Use this to decide Box<FnMut(_, _, _, ...)
                    // After we merge the DomUpdater
                    let arg_count = closure.inputs.len();

                    let add_closure = quote! {
                        #[cfg(target_arch = "wasm32")]
                        {
                          let closure = wasm_bindgen::prelude::Closure::wrap(
                              Box::new(#value) as Box<FnMut(_)>
                          );
                          let closure_rc = std::rc::Rc::new(closure);
                          #var_name_node.as_velement_mut().expect("Not an element")
                              .events.0.insert(#key.to_string(), closure_rc);
                        }
                    };

                    tokens.push(add_closure);
                }
                _ => {
                    let insert_attribute = quote! {
                        #var_name_node.as_velement_mut().expect("Not an element")
                            .attrs.insert(#key.to_string(), #value.to_string());
                    };
                    tokens.push(insert_attribute);
                }
            };
        }

        // The first open tag that we see is our root node so we won't worry about
        // giving it a parent
        if *idx == 0 {
            node_order.push(0);

            if !is_self_closing(&html_tag) {
                parent_stack.push((0, name));
            }

            *idx += 1;
            return;
        }

        let parent_idx = *&parent_stack[parent_stack.len() - 1].0;

        if !is_self_closing(&html_tag) {
            parent_stack.push((*idx, name));
        }
        node_order.push(*idx);

        parent_to_children
            .get_mut(&parent_idx)
            .expect("Parent of this node")
            .push(*idx);

        parent_to_children.insert(*idx, vec![]);

        *idx += 1;
    }
}
