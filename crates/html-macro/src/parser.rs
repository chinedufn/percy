use crate::Tag;
use quote::{quote, quote_spanned};
use std::collections::HashMap;
use syn::export::Span;
use syn::spanned::Spanned;
use syn::{Expr, Ident};

/// Iterate over Tag's that we've parsed and build a tree of VirtualNode's
pub struct HtmlParser {
    /// As we parse our macro tokens we'll generate new tokens to return back into the compiler
    /// when we're done.
    tokens: Vec<proc_macro2::TokenStream>,
    /// Everytime we encounter a new node we'll use the current_idx to name it.
    /// Then we'll increment the current_idx by one.
    /// This gives every node that we encounter a unique name that we can use to find
    /// it later when we want to push child nodes into parent nodes
    current_idx: usize,
    /// The order that we encountered nodes while parsing.
    node_order: Vec<usize>,
    /// Each time we encounter a new node that could possible be a parent node
    /// we push it's node index onto the stack.
    ///
    /// Text nodes cannot be parent nodes.
    parent_stack: Vec<(usize, Ident)>,
    /// Key -> index of the parent node within the HTML tree
    /// Value -> vector of child node indices
    parent_to_children: HashMap<usize, Vec<usize>>,
}

/// TODO: I've hit a good stopping point... but we can clean these methods up / split them up
/// a bit...
impl HtmlParser {
    pub fn new() -> HtmlParser {
        let mut parent_to_children: HashMap<usize, Vec<usize>> = HashMap::new();
        parent_to_children.insert(0, vec![]);

        HtmlParser {
            tokens: vec![],
            current_idx: 0,
            node_order: vec![],
            parent_stack: vec![],
            parent_to_children,
        }
    }

    pub fn push_tag(&mut self, tag: Tag) {
        let idx = &mut self.current_idx;
        let parent_stack = &mut self.parent_stack;
        let node_order = &mut self.node_order;
        let parent_to_children = &mut self.parent_to_children;
        let tokens = &mut self.tokens;

        // TODO: Split each of these into functions and make this DRY. Can barely see what's
        // going on.
        match tag {
            Tag::Open {
                name,
                attrs,
                has_trailing_slash,
            } => {
                // The root node is named `node_0`. All of it's descendants are node_1.. node_2.. etc.
                // This just comes from the `idx` variable
                // TODO: Not sure what the span is supposed to be so I just picked something..
                let var_name_node = Ident::new(format!("node_{}", idx).as_str(), name.span());
                let var_name_element_node = Ident::new(format!("element_node_{}", idx).as_str(), name.span());
                let html_tag = format!("{}", name);

                let element_node = quote! {
                    let mut #var_name_element_node = VirtualNode::element_variant(#html_tag);
                };
                tokens.push(element_node);

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
                                  let closure_rc = Rc::new(closure);
                                  #var_name_element_node.events.0.insert(#key.to_string(), closure_rc);
                                }
                            };

                            tokens.push(add_closure);
                        }
                        _ => {
                            let insert_attribute = quote! {
                                #var_name_element_node.props.insert(#key.to_string(), #value.to_string());
                            };
                            tokens.push(insert_attribute);
                        }
                    };
                }

                let node = quote! {
                    let mut #var_name_node = VirtualNode::Element(#var_name_element_node);
                };
                tokens.push(node);

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
            Tag::Close { name } => {
                let close_span = name.span();
                let close_tag = name.to_string();

                // For example, this should have been <br /> instead of </br>
                if is_self_closing(&close_tag) {
                    let error = format!(
                        r#"{} is a self closing tag. Try "<{}>" or "<{} />""#,
                        close_tag, close_tag, close_tag
                    );
                    let error = quote_spanned! {close_span=> {
                        compile_error!(#error);
                    }};

                    tokens.push(error);
                    return;
                }

                let last_open_tag = parent_stack.pop().expect("Last open tag");

                // TODO: join open and close span. Need to figure out how to enable that.
                //                let open_span = last_open_tag.1.span();

                let last_open_tag = last_open_tag.1.to_string();

                // if div != strong
                if last_open_tag != close_tag {
                    let error = format!(
                        r#"Wrong closing tag. Try changing "{}" into "{}""#,
                        close_tag, last_open_tag
                    );

                    let error = quote_spanned! {close_span=> {
                        compile_error!(#error);
                    }};
                    // TODO: Abort early if we find an error. So we should be returning
                    // a Result.
                    tokens.push(error);
                }
            }
            Tag::Text { text } => {
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
            Tag::Braced { block } => block.stmts.iter().for_each(|stmt| {
                if *idx == 0 {
                    // Here we handle a block being the root node of an `html!` call
                    //
                    // html { { some_node }  }
                    let node = quote! {
                        let node_0 = #stmt;
                    };
                    tokens.push(node);
                } else {
                    // Here we handle a block being a descendant within some html! call
                    //
                    // html { <div> { some_node } </div> }

                    let node_name = format!("node_{}", idx);
                    let node_name = Ident::new(node_name.as_str(), stmt.span());

                    let nodes = quote! {
                        let #node_name = #stmt;
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
            }),
        };
    }

    ///  1. Pop a node off the stack
    ///  2. Look up all of it's children in parent_to_children
    ///  3. Append the children to this node
    ///  4. Move on to the next node (as in, go back to step 1)
    pub fn finish(&mut self) -> proc_macro2::TokenStream {
        let parent_stack = &mut self.parent_stack;
        let node_order = &mut self.node_order;
        let parent_to_children = &mut self.parent_to_children;
        let tokens = &mut self.tokens;

        if node_order.len() > 1 {
            for _ in 0..(node_order.len()) {
                let parent_idx = node_order.pop().unwrap();

                // TODO: Figure out how to really use spans
                let parent_name =
                    Ident::new(format!("node_{}", parent_idx).as_str(), Span::call_site());

                let parent_to_children_indices = match parent_to_children.get(&parent_idx) {
                    Some(children) => children,
                    None => continue,
                };

                if parent_to_children_indices.len() > 0 {
                    for child_idx in parent_to_children_indices.iter() {
                        let children =
                            Ident::new(format!("node_{}", child_idx).as_str(), Span::call_site());

                        let unreachable = quote_spanned!(Span::call_site() => {
                            unreachable!("Non-elements cannot have children");
                        });
                        let push_children = quote! {
                            if let Some(ref mut element_node) =  #parent_name.as_element_variant_mut() {
                                element_node.children.extend(#children.into_iter());
                            } else {
                                #unreachable;
                            }
                        };
                        tokens.push(push_children);
                    }
                }
            }
        }

        // Create a virtual node tree
        let node = quote! {
            {
                #(#tokens)*
                // Root node is always named node_0
                node_0
            }
        };
        node
    }
}

// TODO: Cache this as a HashSet inside of our parser
fn is_self_closing(tag: &str) -> bool {
    let whitelist = [
        "area", "base", "br", "col", "hr", "img", "input", "link", "meta", "param", "command",
        "keygen", "source",
    ];

    for whitelisted in whitelist.iter() {
        if &tag == whitelisted {
            return true;
        }
    }

    return false;
}
