//! Tests that ensure that we create the right DOM element from a VirtualNode
//!
//! To run all tests in this file:
//!
//! wasm-pack test --chrome --headless crates/percy-dom --test create_dom_node

extern crate wasm_bindgen_test;
extern crate web_sys;
use percy_dom::event::EventsByNodeIdx;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::Element;

use percy_dom::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

/// wasm-pack test --chrome --headless crates/percy-dom --test create_dom_node -- nested_divs
#[wasm_bindgen_test]
fn nested_divs() {
    let vdiv = html! { <div> <div> <div></div> </div> </div> };
    let div: Element = vdiv
        .create_dom_node(0, &mut EventsByNodeIdx::new())
        .unchecked_into();

    assert_eq!(&div.inner_html(), "<div><div></div></div>");
}

// wasm-pack test --chrome --headless crates/percy-dom --test create_dom_node -- svg_element
// TODO (June 2019): Temporarily disabled until we figure out why it's failing in CI but not
//  failing locally
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

/// wasm-pack test --chrome --headless crates/percy-dom --test create_dom_node -- div_with_attributes
#[wasm_bindgen_test]
fn div_with_attributes() {
    let vdiv = html! { <div id="id-here" class="two classes"></div> };
    let div: Element = vdiv
        .create_dom_node(0, &mut EventsByNodeIdx::new())
        .unchecked_into();

    assert_eq!(&div.id(), "id-here");

    assert!(div.class_list().contains("two"));
    assert!(div.class_list().contains("classes"));

    assert_eq!(div.class_list().length(), 2);
}
