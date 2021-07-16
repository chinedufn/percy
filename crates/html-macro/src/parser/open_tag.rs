use crate::parser::{is_self_closing, is_valid_tag, HtmlParser};
use crate::tag::Attr;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::__private::TokenStream2;
use syn::{Expr, ExprClosure};

impl HtmlParser {
    /// Parse an incoming Tag::Open
    pub(crate) fn parse_open_tag(
        &mut self,
        name: &Ident,
        closing_span: &Span,
        attrs: &Vec<Attr>,
        is_self_closing_tag: bool,
    ) {
        self.set_most_recent_open_tag_end(closing_span.clone());

        let idx = &mut self.current_node_idx;
        let parent_to_children = &mut self.parent_to_children;
        let parent_stack = &mut self.parent_stack;
        let tokens = &mut self.tokens;
        let node_order = &mut self.node_order;

        // The root node is named `node_0`. All of it's descendants are node_1.. node_2.. etc.
        // This just comes from the `idx` variable
        // TODO: Not sure what the span is supposed to be so I just picked something..
        let var_name_node = Ident::new(format!("node_{}", idx).as_str(), name.span());
        let html_tag = format!("{}", name);
        let is_html_tag = is_valid_tag(&html_tag);

        if is_html_tag {
            create_valid_node(&html_tag, attrs, &var_name_node, tokens);
        } else if !html_tag.chars().next().unwrap().is_uppercase() {
            let compile_err = invalid_tag_compile_error(name, &html_tag, &var_name_node);
            tokens.push(compile_err);
        } else {
            let node = component_node(*idx, name, &html_tag, attrs, &var_name_node);
            tokens.push(node);
        }

        // The first open tag that we see is our root node so we won't worry about
        // giving it a parent
        if *idx == 0 {
            node_order.push(0);

            if !is_self_closing(&html_tag) && !is_self_closing_tag {
                parent_stack.push((0, name.clone()));
            }

            *idx += 1;
            return;
        }

        let parent_idx = *&parent_stack[parent_stack.len() - 1].0;

        if !is_self_closing(&html_tag) && !is_self_closing_tag {
            parent_stack.push((*idx, name.clone()));
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

fn create_valid_node(
    html_tag: &str,
    attrs: &Vec<Attr>,
    var_name_node: &Ident,
    tokens: &mut Vec<TokenStream2>,
) {
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
                let _arg_count = closure.inputs.len();

                #[cfg(target_arch = "wasm32")]
                {
                    let add_closure = quote! {
                        {
                            let closure = Closure::wrap(
                                Box::new(#value) as Box<FnMut(_)>
                            );
                            let closure_rc = std::rc::Rc::new(closure);
                            #var_name_node.as_velement_mut().expect("Not an element")
                                .events.0.insert(#key.to_string(), closure_rc);
                        }
                    };

                    tokens.push(add_closure);
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    if key == "oninput" || key == "onclick" {
                        let add_event = simulated_event_tokens(var_name_node, closure, &attr.key);
                        tokens.push(add_event);
                    }
                }
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
}

fn invalid_tag_compile_error(name: &Ident, html_tag: &str, var_name_node: &Ident) -> TokenStream {
    let error = format!(
        r#"{} is not a valid HTML tag.
                
If you are trying to use a valid HTML tag, perhaps there's a typo?
                
If you are trying to use a custom component, please capitalize the component name.
                
custom components: https://chinedufn.github.io/percy/html-macro/custom-components/index.html"#,
        html_tag,
    );
    let span = name.span();
    let invalid_tag_name_error = quote_spanned! {span=> {
        compile_error!(#error);
    }};

    let compile_err = quote! {
        #invalid_tag_name_error
        let mut #var_name_node = VirtualNode::text("error");
    };

    compile_err
}

fn component_node(
    idx: usize,
    name: &Ident,
    html_tag: &str,
    attrs: &Vec<Attr>,
    var_name_node: &Ident,
) -> TokenStream {
    let var_name_component = Ident::new(format!("component_{}", idx).as_str(), name.span());
    let component_ident = Ident::new(format!("{}", html_tag).as_str(), name.span());

    let component_props: Vec<proc_macro2::TokenStream> = attrs
        .into_iter()
        .map(|attr| {
            let key = Ident::new(format!("{}", attr.key).as_str(), name.span());
            let value = &attr.value;

            quote! {
                #key: #value,
            }
        })
        .collect();

    let node = quote! {
        let mut #var_name_component = #component_ident { #(#component_props),* };
        let mut #var_name_node = #var_name_component.render();
    };

    node
}

// ```
// // Example:
//
// html! { <button oninput=|_: DomInputEvent| {}> Hi </button>
//
// ... closure becomes ...
//
// let node_5.as_velement_mut().unwrap()
//   .known_events.as_mut().unwrap()
//   .oninput.borrow_mut() = Some(Box::new(CLOSURE_HERE) as Box<_>);
// ```
fn simulated_event_tokens(
    node_name: &Ident,
    closure: &ExprClosure,
    event_name: &Ident,
) -> TokenStream {
    quote! {
        if #node_name.as_velement_mut().expect("Not an element")
            .known_events.is_none() {
          #node_name.as_velement_mut().unwrap()
            .known_events = Some(KnownEvents::new());
        }

          *(#node_name.as_velement_mut().unwrap()
            .known_events
            .as_mut()
            .unwrap()
            .#event_name.borrow_mut()) =
            Some(Box::new(#closure) as Box<_>);
    }
}
