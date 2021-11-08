//! Tests that ensure that we create the right DOM element from a VirtualNode
//!
//! To run all tests in this file:
//!
//! wasm-pack test --chrome --headless crates/percy-dom --test dangerous_inner_html

extern crate wasm_bindgen_test;
extern crate web_sys;
use percy_dom::event::EventsByNodeIdx;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::Element;

use percy_dom::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

/// wasm-pack test --chrome --headless crates/percy-dom --test dangerous_inner_html -- new_elem_inner_html
#[wasm_bindgen_test]
fn new_elem_inner_html() {
    let mut div: VirtualNode = html! {
    <div></div>
    };
    div.as_velement_mut()
        .unwrap()
        .special_attributes
        .dangerous_inner_html = Some("<span>hi</span>".to_string());

    let div: Element = div
        .create_dom_node(0, &mut EventsByNodeIdx::new())
        .unchecked_into();

    assert_eq!(div.inner_html(), "<span>hi</span>");
}

/// Verify that if we patch a node with dangerous_inner_html over another node that has
/// dangerous_inner_html we overwrite the innerHTML.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test dangerous_inner_html -- inner_html_overwrite
#[wasm_bindgen_test]
fn inner_html_overwrite() {
    let mut start: VirtualNode = VirtualNode::element("div");
    start
        .as_velement_mut()
        .unwrap()
        .special_attributes
        .dangerous_inner_html = Some("<span>OLD</span>".to_string());

    let mut end: VirtualNode = VirtualNode::element("div");
    end.as_velement_mut()
        .unwrap()
        .special_attributes
        .dangerous_inner_html = Some("<span>NEW</span>".to_string());

    let div = start.create_dom_node(0, &mut EventsByNodeIdx::new());

    let patches = percy_dom::diff(&start, &end);
    percy_dom::patch(div.clone(), &end, &mut EventsByNodeIdx::new(), &patches).unwrap();

    let div: Element = div.unchecked_into();
    assert_eq!(div.inner_html(), "<span>NEW</span>");
}

/// Verify that if the old node has dangerous_inner_html but the new node does not, the
/// dangerous_inner_html is removed.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test dangerous_inner_html -- remove_inner_html
#[wasm_bindgen_test]
fn remove_inner_html() {
    let mut start: VirtualNode = VirtualNode::element("div");
    start
        .as_velement_mut()
        .unwrap()
        .special_attributes
        .dangerous_inner_html = Some("<span>OLD</span>".to_string());

    let end: VirtualNode = VirtualNode::element("div");

    let div = start.create_dom_node(0, &mut EventsByNodeIdx::new());

    let patches = percy_dom::diff(&start, &end);
    percy_dom::patch(div.clone(), &end, &mut EventsByNodeIdx::new(), &patches).unwrap();

    let div: Element = div.unchecked_into();
    assert_eq!(div.inner_html(), "");
}
