extern crate wasm_bindgen_test;
extern crate web_sys;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen_test::*;

use web_sys::*;

#[macro_use]
extern crate virtual_dom_rs;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn nested_divs() {
    let div = html! { <div> <div> <div></div> </div> </div> };
    let div = div.create_element();

    assert_eq!(&div.inner_html(), "<div><div></div></div>");
}

#[wasm_bindgen_test]
fn div_with_properties() {
    let mut div = html! { <div id="id-here", class="two classes",></div> };
    let div = div.create_element();

    assert_eq!(&div.id(), "id-here");

    assert!(div.class_list().contains("two"));;
    assert!(div.class_list().contains("classes"));;

    assert_eq!(div.class_list().length(), 2);
}

#[wasm_bindgen_test]
fn click_event() {
    let clicked = Rc::new(Cell::new(false));
    let clicked_clone = Rc::clone(&clicked);

    let div = html! {
     <div
         !onclick=move |_ev: MouseEvent| {
             clicked_clone.set(true);
         },
     >
     </div>
    };

    let click_event = Event::new("click").unwrap();

    let div = div.create_element().element;

    (web_sys::EventTarget::from(div))
        .dispatch_event(&click_event)
        .unwrap();

    assert_eq!(*clicked, Cell::new(true));
}

