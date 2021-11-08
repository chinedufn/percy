use crate::tag::Attr;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::__private::TokenStream2;
use syn::{Expr, ExprClosure, Pat, PatType, Type};

// This only gets used in cases where we're generating a compile time error for not using a key..
// So the user will never see this.. We just need any string.
const FAKE_KEY: &'static str = "...";

// Create the tokens that insert a closure into the virtual element.
//
// Tests can be found in crates/html-macro-test/src/events.rs
pub(super) fn insert_closure_tokens(
    var_name_node: &Ident,
    event_attribute: &Attr,
    closure: &ExprClosure,
    key_attr_value: Option<&Expr>,
) -> TokenStream {
    let arg_count = closure.inputs.len();
    let event_name = event_attribute.key.to_string();

    let attr_key_span = &event_attribute.key.span();

    // TODO: Refactor duplicate code between these blocks.
    if event_name == "on_create_element" {
        let mut maybe_missing_key_error = None;

        let key_attr_value = if let Some(key_attr_value) = key_attr_value {
            quote! {
                #key_attr_value
            }
        } else {
            let error = format!(
                r#"Whenever you use the `on_create_element=...` attribute,
you must also use must use the `key="..."` attribute.

Documentation:
  -> https://chinedufn.github.io/percy/html-macro/real-elements-and-nodes/on-create-elem/index.html
            "#
            );

            maybe_missing_key_error = Some(quote_spanned! { attr_key_span.clone() => {
                compile_error!(#error);
            }});

            quote! { #FAKE_KEY }
        };

        let closure =
            maybe_set_arg_type(closure, quote! { __html_macro_helpers__::web_sys::Element });

        let setter = if arg_count == 0 {
            quote! { set_on_create_element_no_args }
        } else {
            quote! { set_on_create_element }
        };

        quote! {
            let event_callback = #closure;

            #var_name_node.as_velement_mut().unwrap()
              .special_attributes.#setter(#key_attr_value.to_string(), event_callback);

            #maybe_missing_key_error
        }
    } else if event_name == "on_remove_element" {
        let mut maybe_missing_key_error = None;

        let key_attr_value = if let Some(key_attr_value) = key_attr_value {
            quote! {
                #key_attr_value
            }
        } else {
            let error = format!(
                r#"Whenever you use the `on_remove_element=...` attribute,
you must also use must use the `key="..."` attribute.

Documentation:
  -> https://chinedufn.github.io/percy/html-macro/real-elements-and-nodes/on-remove-elem/index.html
            "#
            );

            maybe_missing_key_error = Some(quote_spanned! { attr_key_span.clone() => {
                compile_error!(#error);
            }});

            quote! { #FAKE_KEY }
        };

        let closure =
            maybe_set_arg_type(closure, quote! { __html_macro_helpers__::web_sys::Element });

        let setter = if arg_count == 0 {
            quote! { set_on_remove_element_no_args }
        } else {
            quote! { set_on_remove_element }
        };

        quote! {
            let event_callback = #closure;

            #var_name_node.as_velement_mut().unwrap()
              .special_attributes.#setter(#key_attr_value.to_string(), event_callback);

            #maybe_missing_key_error
        }
    } else if arg_count == 0 {
        quote! {
            let event_callback = #closure;
            #var_name_node.as_velement_mut().unwrap().events.insert_no_args(
                #event_name.into(),
                std::rc::Rc::new(
                    std::cell::RefCell::new( event_callback )
                )
            );
        }
    } else if event_name == "onclick" {
        let tokens = quote! {
            #var_name_node.as_velement_mut().unwrap().events.insert_mouse_event(
                #event_name.into(),
                std::rc::Rc::new(
                    std::cell::RefCell::new( event_callback )
                )
            );
        };

        let tokens = if arg_count == 0 {
            quote! {
                let event_callback = #closure;
                #tokens
            }
        } else {
            let closure = maybe_set_arg_type(
                closure,
                quote! { __html_macro_helpers__::event::MouseEvent },
            );

            quote! {
                let event_callback = #closure;
                #tokens
            }
        };

        tokens
    } else {
        let arg_type_placeholders: Vec<TokenStream2> =
            (0..arg_count).into_iter().map(|_| quote! { _ }).collect();

        quote! {
          #[cfg(target_arch = "wasm32")]
          {

              let closure = Closure::wrap(
                  Box::new(#closure) as Box<dyn FnMut(#(#arg_type_placeholders)*)>
              );
              let closure_rc = std::rc::Rc::new(closure);

              #var_name_node.as_velement_mut().unwrap()
                  .events.__insert_unsupported_signature(#event_name.into(), closure_rc);
          }

          #[cfg(not(target_arch = "wasm32"))]
          {
              // Ensures that the variables that the closure captures are considered used.
              let _ = #closure;
          }
        }
    }
}

/// Clone the incoming closure tokens.
///
/// - If the closure does not have any arguments, return the closure.
///
/// - If the closure's first argument has a type annotation, return the closure.
///
/// - If the closure's first argument does not have a type annotation, add a type annotation to
///   it then return the closure.
///
/// ## Examples
///
/// Say we have the type `some::CoolType`
///
/// `|| {}` -> BECOMES -> `|| {}`
///
/// `|foo| {}` -> BECOMES -> `|foo: some::CoolType| {}`
///
/// `|foo: Bar| {}` -> BECOMES -> `|foo: Bar| {}`
fn maybe_set_arg_type(closure: &ExprClosure, ty: TokenStream2) -> ExprClosure {
    let mut closure = closure.clone();

    if closure.inputs.len() == 0 {
        return closure;
    }

    let arg0 = closure.inputs.first_mut().unwrap();

    if let Pat::Ident(ident) = arg0 {
        // Add the type to the closure to avoid `type annotations needed` errors.
        // Example:
        //   Start: |arg| {}
        //   End: |arg: __private__event::MouseEvent| {}

        let ident = Pat::Ident(ident.clone());

        *arg0 = Pat::Type(PatType {
            attrs: vec![],
            pat: Box::new(ident),
            colon_token: Default::default(),
            ty: Box::new(Type::Verbatim(ty)),
        });
    }

    closure
}
