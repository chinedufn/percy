//! Test the on remove element special attribute.
//!
//! To run all tests in this file:
//!
//! wasm-pack test --chrome --headless crates/percy-dom --test on_remove_element

extern crate wasm_bindgen_test;
extern crate web_sys;

use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen_test::*;

use crate::testing_utilities::{create_mount, get_element_by_id, random_id};
use percy_dom::prelude::*;

mod testing_utilities;

wasm_bindgen_test_configure!(run_in_browser);

/// Verify that when we remove an element by patching over it we call it's on_remove_elem function.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test on_remove_element -- remove_node
#[wasm_bindgen_test]
fn remove_node() {
    let old_elem_id = random_id();

    let called = Rc::new(Cell::new(false));
    let called_clone = called.clone();

    let mut old = html! { <div id=old_elem_id> </div> };
    old.as_velement_mut()
        .unwrap()
        .special_attributes
        .set_on_remove_element("foo", move |elem: web_sys::Element| {
            assert_eq!(elem.id(), old_elem_id);
            called_clone.set(true);
        });

    let new = VirtualNode::element("span");

    let mount = create_mount();
    let mut pdom = PercyDom::new_append_to_mount(old, &mount);
    pdom.update(new);

    assert!(called.get());
}

/// Verify that if an element is removed and it's children have on remove element callbacks,
/// they get called.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test on_remove_element -- on_remove_elem_called_on_children
#[wasm_bindgen_test]
fn on_remove_elem_called_on_children() {
    let child_id = random_id();

    let called = Rc::new(Cell::new(false));
    let called_clone = called.clone();

    let mut child = html! {
         <em id = child_id></em>
    };
    child
        .as_velement_mut()
        .unwrap()
        .special_attributes
        .set_on_remove_element("foo", move |elem: web_sys::Element| {
            assert_eq!(elem.id(), child_id);
            called_clone.set(true);
        });

    let old = html! {
        <div>{ child }</div>
    };
    let new = VirtualNode::element("span");

    let mount = create_mount();
    let mut pdom = PercyDom::new_append_to_mount(old, &mount);

    assert_eq!(called.get(), false);
    pdom.update(new);
    assert_eq!(called.get(), true);
}
