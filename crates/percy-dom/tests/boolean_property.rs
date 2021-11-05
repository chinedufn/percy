//! Verify that when we set the value attribute for an input or textarea element we also call
//! .set_value(value) on the element. Without this the `.value()` method on the element won't
//! return the new value.
//!
//! To run all tests in this file:
//! wasm-pack test --chrome --headless crates/percy-dom --test boolean_property

use wasm_bindgen_test::*;

use wasm_bindgen::JsCast;
use web_sys::*;

use percy_dom::event::EventsByNodeIdx;
use percy_dom::{Patch, VirtualNode};
use std::collections::HashMap;
use virtual_node::{AttributeValue, VElement};

wasm_bindgen_test_configure!(run_in_browser);

/// Verify that we create a node with the attribute if the boolean property is true.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test boolean_property -- create_elem_boolean_property_true
#[wasm_bindgen_test]
fn create_elem_boolean_property_true() {
    let mut elem = VElement::new("button");
    elem.attrs
        .insert("disabled".to_string(), AttributeValue::Bool(true));

    let node: VirtualNode = elem.into();
    let node = node.create_dom_node(0, &mut EventsByNodeIdx::new());
    assert!(node_as_button(&node).disabled());
}

/// Verify that we do not create a node with an attribute if the boolean property is false.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test boolean_property -- create_elem_boolean_property_false
#[wasm_bindgen_test]
fn create_elem_boolean_property_false() {
    let mut elem = VElement::new("button");
    elem.attrs
        .insert("disabled".to_string(), AttributeValue::Bool(false));

    let node: VirtualNode = elem.into();
    let node = node.create_dom_node(0, &mut EventsByNodeIdx::new());
    assert!(!node_as_button(&node).disabled());
}

/// Verify that if we patch a node's boolean attribute to true it gets added.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test boolean_property -- patch_elem_boolean_property_true
#[wasm_bindgen_test]
fn patch_elem_boolean_property_true() {
    let elem: VirtualNode = VElement::new("button").into();

    let node = elem.create_dom_node(0, &mut EventsByNodeIdx::new());

    let mut attributes = HashMap::new();
    let true_attribute = AttributeValue::Bool(true);
    attributes.insert("disabled", &true_attribute);
    let patch = Patch::AddAttributes(0, attributes);
    percy_dom::patch(
        node.clone(),
        &VirtualNode::element("..."),
        &mut EventsByNodeIdx::new(),
        &vec![patch],
    )
    .unwrap();

    assert!(node_as_button(&node).disabled());
}

/// Verify that if we patch a node's boolean attribute to false it gets removed.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test boolean_property -- patch_elem_boolean_property_false
#[wasm_bindgen_test]
fn patch_elem_boolean_property_false() {
    let elem: VirtualNode = VElement::new("button").into();

    let node = elem.create_dom_node(0, &mut EventsByNodeIdx::new());
    node.dyn_ref::<HtmlButtonElement>()
        .unwrap()
        .set_disabled(true);
    assert!(node_as_button(&node).disabled());

    let mut attributes = HashMap::new();
    let false_attribute = AttributeValue::Bool(false);
    attributes.insert("disabled", &false_attribute);
    let patch = Patch::AddAttributes(0, attributes);
    percy_dom::patch(
        node.clone(),
        &VirtualNode::element(".."),
        &mut EventsByNodeIdx::new(),
        &vec![patch],
    )
    .unwrap();

    assert!(!node_as_button(&node).disabled());
}

fn node_as_button(elem: &Node) -> &HtmlButtonElement {
    elem.dyn_ref::<HtmlButtonElement>().unwrap()
}
