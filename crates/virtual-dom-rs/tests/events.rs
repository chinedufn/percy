extern crate wasm_bindgen_test;
extern crate web_sys;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen_test::*;

use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use web_sys::*;

#[macro_use]
extern crate virtual_dom_rs;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn on_input() {
    let text = Rc::new(RefCell::new("".to_string()));
    let text_clone = Rc::clone(&text);

    let input = html! {
     <input
         oninput=|input_event: InputEvent| {
//            let input_text = ((input_event.as_ref() as Event).target().unwrap().as_ref() as HtmlInputElement).value();
//             *text_clone.borrow_mut() = input_text;
         },
     >
     </input>
    };

    let input_event = InputEvent::new("input").unwrap();
    let input = input.create_element();

    (web_sys::EventTarget::from(input))
        .dispatch_event(input_event.as_ref() as &web_sys::Event)
        .unwrap();

    assert_eq!(&*text.borrow(), "hello world");
}
