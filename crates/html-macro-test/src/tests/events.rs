use crate::tests::all_tests::HtmlMacroTest;
use html_macro::html;
use percy_dom::event::EventHandler;
use virtual_node::VirtualNode;

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
