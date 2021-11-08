use crate::tests::all_tests::HtmlMacroTest;
use percy_dom::event::EventHandler;
use percy_dom::prelude::*;

/// Unsupported events that have arguments are wrapped using `wasm_bindgen::Closure::wrap`.
///
/// This doesn't work on non-wasm32 targets, so our macro ignores them in non wasm32 targets.
#[test]
fn ignore_unsupported_events_on_non_wasm32_targets_if_they_have_args() {
    HtmlMacroTest {
        generated: html! {
            <div on_some_unsupported_event = |_: u8| {} ></div>
        },
        expected: html! {<div></div>},
    }
    .test();
}

/// Events that don't have arguments are always stored as EventHandler::NoArgs.
#[test]
fn store_unsupported_events_on_non_wasm32_targets_if_no_args() {
    let node: VirtualNode = html! {
        <div on_some_unsupported_event = || {} ></div>
    };
    let handler = node
        .as_velement_ref()
        .unwrap()
        .events
        .get(&"on_some_unsupported_event".into())
        .unwrap();
    assert!(matches!(handler, EventHandler::NoArgs(_)));
}

/// We don't store unsupported events in non wasm32 targets.. but we still want the variables that
/// these events capture to be considered used.
/// We test this using `[#deny(unused)]` on the variable.
#[test]
fn closure_moved_variables_used() {
    #[deny(unused)]
    let moved_var = ();

    html! {
        <button on_some_unsupported_event=move |_: u8| {let _ = moved_var;}></button>
    };
}

/// Verify that onclick events are stored.
#[test]
fn stores_onclick_events() {
    let node: VirtualNode = html! {
        <button onclick = |_: virtual_node::event::MouseEvent| {}> </button>
    };

    let event = node
        .as_velement_ref()
        .unwrap()
        .events
        .get(&"onclick".into())
        .unwrap();
    assert!(matches!(event, EventHandler::MouseEvent(_)));
}

/// Verify that we can set the on create element function.
#[test]
fn on_create_element() {
    let node: VirtualNode = html! {
        <div key = "my-key" on_create_element=||{}> </div>
    };
    assert_eq!(
        node.as_velement_ref()
            .unwrap()
            .special_attributes
            .on_create_element_key(),
        Some(&"my-key".into())
    );
}

/// Verify that we can set the on remove element function.
#[test]
fn on_remove_element() {
    let node: VirtualNode = html! {
        <div key = "my-key" on_remove_element=||{}> </div>
    };
    assert_eq!(
        node.as_velement_ref()
            .unwrap()
            .special_attributes
            .on_remove_element_key(),
        Some(&"my-key".into())
    );
}

/// Verify that we do not need to provide the type for events that we support.
///
/// We make use of the passed in type inside each closure.
/// If the test compile then we know that we did not get a `type annotations needed` error.
#[test]
fn eliding_event_arg_type() {
    html! {
        <div
          key="123"

          onclick = |event| {
            event.stop_propagation();
          }
          on_create_element = |element| {
            element.id();
          }
          on_remove_element = |element| {
            element.id();
          }
          // TODO: As we add more supported events to `open_tag.rs` we can add handlers for them here.
        >
        </div>
    };
}
