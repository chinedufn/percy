//! Tests that ensure that we create the right DOM element from a VirtualNode
//!
//! To run all tests in this file:
//!
//! wasm-pack test --chrome --headless crates/percy-dom --test create_element

extern crate wasm_bindgen_test;
extern crate web_sys;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Element, Event, EventTarget, MouseEvent};

use percy_dom::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

/// wasm-pack test --chrome --headless crates/percy-dom --test create_element -- nested_divs
#[wasm_bindgen_test]
fn nested_divs() {
    let vdiv = html! { <div> <div> <div></div> </div> </div> };
    let div: Element = vdiv.create_dom_node().node.unchecked_into();

    assert_eq!(&div.inner_html(), "<div><div></div></div>");
}

/// wasm-pack test --chrome --headless crates/percy-dom --test create_element -- svg_element
/// TODO: Temporarily disabled until we figure out why it's failing in CI but not failing locally
// #[wasm_bindgen_test]
// fn svg_element() {
//     let vdiv = html! { <div><svg xmlns="http://www.w3.org/2000/svg">
//       <circle cx="50" cy="50" r="50"/>
//     </svg></div> };
//     let div: Element = vdiv.create_dom_node().node.unchecked_into();

//     assert_eq!(
//         &div.inner_html(),
//         r#"<svg xmlns="http://www.w3.org/2000/svg"><circle cx="50" cy="50" r="50"></circle></svg>"#
//     );
// }

/// wasm-pack test --chrome --headless crates/percy-dom --test create_element -- div_with_attributes
#[wasm_bindgen_test]
fn div_with_attributes() {
    let vdiv = html! { <div id="id-here" class="two classes"></div> };
    let div: Element = vdiv.create_dom_node().node.unchecked_into();

    assert_eq!(&div.id(), "id-here");

    assert!(div.class_list().contains("two"));
    assert!(div.class_list().contains("classes"));

    assert_eq!(div.class_list().length(), 2);
}

/// wasm-pack test --chrome --headless crates/percy-dom --test create_element -- click_event
#[wasm_bindgen_test]
fn click_event() {
    let clicked = Rc::new(Cell::new(false));
    let clicked_clone = Rc::clone(&clicked);

    let div = html! {
     <div
         onclick=move |_ev: MouseEvent| {
             clicked_clone.set(true);
         }
     >
     </div>
    };

    let click_event = Event::new("click").unwrap();

    let div = div.create_dom_node().node;

    (EventTarget::from(div))
        .dispatch_event(&click_event)
        .unwrap();

    assert_eq!(*clicked, Cell::new(true));
}

/// Verify that when we create a new element we call it's on_create_elem function.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test create_element -- on_create_elem_new_node
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
        .on_create_elem = Some((
        0,
        wrap_closure(move |elem: web_sys::Element| {
            elem.set_inner_html("Hello world");
        }),
    ));

    let div: Element = div.create_dom_node().node.unchecked_into();

    assert_eq!(div.inner_html(), "Hello world");
}

/// Verify that if we are patching over another element that does not have an on_create_elem
/// attribute we call the new node's on_create_elem.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test create_element -- on_create_elem_triggered_via_patch
#[wasm_bindgen_test]
fn on_create_elem_triggered_via_patch() {
    let start = VirtualNode::element("div");

    let mut end: VirtualNode = VirtualNode::element("div");
    end.as_velement_mut()
        .unwrap()
        .special_attributes
        .on_create_elem = Some((
        0,
        wrap_closure(move |elem: web_sys::Element| {
            elem.set_inner_html("Hello world");
        }),
    ));

    let div = start.create_dom_node();

    let patches = percy_dom::diff(&start, &end);
    percy_dom::patch(div.node.clone(), &patches).unwrap();

    let div: Element = div.node.unchecked_into();
    assert_eq!(div.inner_html(), "Hello world");
}

/// Verify that if we are patching over another element that has the same on_create_elem ID
/// (i.e., we're probably patching over the same virtual-node that's been slightly changed..)
/// we do not call the new node's on_create_elem.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test create_element -- on_create_elem_not_triggered_via_patch_if_same_id
#[wasm_bindgen_test]
fn on_create_elem_not_triggered_via_patch_if_same_id() {
    let mut start = html! {<div id="original"></div>};
    start
        .as_velement_mut()
        .unwrap()
        .special_attributes
        .on_create_elem = Some((0, wrap_closure(|_elem: web_sys::Element| {})));

    let mut end: VirtualNode = html! {<div id="new"></div>};
    end.as_velement_mut()
        .unwrap()
        .special_attributes
        .on_create_elem = Some((
        0,
        wrap_closure(move |elem: web_sys::Element| {
            panic!("CLOSURE SHOULD NOT GET CALLED");
        }),
    ));

    let div = start.create_dom_node();

    let patches = percy_dom::diff(&start, &end);
    percy_dom::patch(div.node.clone(), &patches).unwrap();

    let div: Element = div.node.unchecked_into();
    assert_eq!(div.id(), "new");
}
