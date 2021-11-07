//! Test the on create element special attribute.
//!
//! To run all tests in this file:
//!
//! wasm-pack test --chrome --headless crates/percy-dom --test on_create_element

extern crate wasm_bindgen_test;
extern crate web_sys;
use percy_dom::event::EventsByNodeIdx;
use testing_utilities::random_id;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::Element;

use percy_dom::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

mod testing_utilities;

/// Verify that when we create a new element we call it's on_create_elem function.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test on_create_element -- on_create_elem_new_node
#[wasm_bindgen_test]
fn on_create_elem_new_node() {
    let mut div: VirtualNode = html! {
    <div>
        <span>This span should get replaced</span>
    </div>
    };

    div.as_velement_mut()
        .unwrap()
        .special_attributes
        .set_on_create_element("foo", move |elem: web_sys::Element| {
            elem.set_inner_html("Hello world");
        });

    let div: Element = div
        .create_dom_node(0, &mut EventsByNodeIdx::new())
        .unchecked_into();

    assert_eq!(div.inner_html(), "Hello world");
}

/// Verify that if we are patching over an old element that does not have an on_create_elem,
/// attribute we call the new node's on_create_elem.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test on_create_element -- on_create_elem_triggered_via_patch
#[wasm_bindgen_test]
fn on_create_elem_triggered_via_patch() {
    let start = VirtualNode::element("div");

    let end_id = random_id();
    let mut end: VirtualNode = html! { <div id=end_id> </div>};
    end.as_velement_mut()
        .unwrap()
        .special_attributes
        .set_on_create_element("foo", move |elem: web_sys::Element| {
            assert_eq!(elem.id(), end_id);
            elem.set_inner_html("Hello world");
        });

    let mut events = EventsByNodeIdx::new();
    let div = start.create_dom_node(0, &mut events);

    let patches = percy_dom::diff(&start, &end);
    percy_dom::patch(div.clone(), &end, &mut events, &patches).unwrap();

    let div: Element = div.unchecked_into();
    assert_eq!(div.inner_html(), "Hello world");
}

/// Verify that if we are patching over another element that has the same on_create_elem ID
/// (i.e., we're probably patching over the same virtual-node that's been slightly changed..)
/// we do not call the new node's on_create_elem.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test on_create_element -- on_create_elem_not_triggered_via_patch_if_same_id
#[wasm_bindgen_test]
fn on_create_elem_not_triggered_via_patch_if_same_id() {
    let mut start = html! {<div id="original"></div>};
    start
        .as_velement_mut()
        .unwrap()
        .special_attributes
        .set_on_create_element("same-key", |_elem: web_sys::Element| {});

    let mut end: VirtualNode = html! {<div id="new"></div>};
    end.as_velement_mut()
        .unwrap()
        .special_attributes
        .set_on_create_element("same-key", move |_elem: web_sys::Element| {
            panic!("On create element function should not have gotten called.");
        });

    let mut events = EventsByNodeIdx::new();
    let div = start.create_dom_node(0, &mut events);

    let patches = percy_dom::diff(&start, &end);
    percy_dom::patch(div.clone(), &end, &mut events, &patches).unwrap();

    let div: Element = div.unchecked_into();
    assert_eq!(div.id(), "new");
}
