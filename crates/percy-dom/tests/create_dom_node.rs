//! Tests that ensure that we create the right DOM element from a VirtualNode
//!
//! To run all tests in this file:
//!
//! CHROMEDRIVER=chromedriver cargo test -p percy-dom --target wasm32-unknown-unknown --test create_dom_node

extern crate wasm_bindgen_test;
extern crate web_sys;
use percy_dom::event::VirtualEvents;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::Element;

use percy_dom::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

/// CHROMEDRIVER=chromedriver cargo test -p percy-dom --target wasm32-unknown-unknown --test create_dom_node -- nested_divs
#[wasm_bindgen_test]
fn nested_divs() {
    let vdiv = html! { <div> <div> <div></div> </div> </div> };
    let div: Element = vdiv
        .create_dom_node(&mut VirtualEvents::new())
        .0
        .unchecked_into();

    assert_eq!(&div.inner_html(), "<div><div></div></div>");
}

// CHROMEDRIVER=chromedriver cargo test -p percy-dom --target wasm32-unknown-unknown --test create_dom_node -- svg_element
#[wasm_bindgen_test]
fn svg_element() {
    let vdiv = html! { <div><svg xmlns="http://www.w3.org/2000/svg">
      <circle cx="50" cy="60" r="70"/>
    </svg></div> };
    let div: Element = vdiv
        .create_dom_node(&mut VirtualEvents::new())
        .0
        .unchecked_into();

    let svg = div.first_element_child().unwrap();
    assert_eq!(svg.namespace_uri().unwrap(), "http://www.w3.org/2000/svg");

    let circle = svg.first_element_child().unwrap();
    assert_eq!(
        circle.namespace_uri().unwrap(),
        "http://www.w3.org/2000/svg"
    );
    assert_eq!(circle.tag_name(), "circle");
    assert_eq!(circle.get_attribute("cx").unwrap(), "50");
    assert_eq!(circle.get_attribute("cy").unwrap(), "60");
    assert_eq!(circle.get_attribute("r").unwrap(), "70");
}

/// CHROMEDRIVER=chromedriver cargo test -p percy-dom --target wasm32-unknown-unknown --test create_dom_node -- div_with_attributes
#[wasm_bindgen_test]
fn div_with_attributes() {
    let vdiv = html! { <div id="id-here" class="two classes"></div> };
    let div: Element = vdiv
        .create_dom_node(&mut VirtualEvents::new())
        .0
        .unchecked_into();

    assert_eq!(&div.id(), "id-here");

    assert!(div.class_list().contains("two"));
    assert!(div.class_list().contains("classes"));

    assert_eq!(div.class_list().length(), 2);
}
