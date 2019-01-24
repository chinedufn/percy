//! Ensure that our DomUpdater maintains Rc's to closures so that they work even
//! after dropping virtual dom nodes.

use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use virtual_dom_rs::prelude::*;
use virtual_dom_rs::recurse_html;
use virtual_dom_rs::html;
use wasm_bindgen::JsCast;
use wasm_bindgen_test;
use wasm_bindgen_test::*;
use web_sys::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn closure_not_dropped() {
    let text = Rc::new(RefCell::new("Start Text".to_string()));
    let text_clone = Rc::clone(&text);

    let document = web_sys::window().unwrap().document().unwrap();

    {
        let input = html! {
         <input
            id="input-elem",
             // On input we'll set our Rc<RefCell<String>> value to the input elements value
             !oninput=move |event: Event| {
                let input_elem = event.target().unwrap();
                let input_elem = input_elem.dyn_into::<HtmlInputElement>().unwrap();
                *text_clone.borrow_mut() = input_elem.value();
             },
             value="End Text",
         >
         </input>
        };

        let input = input.create_element();

        document.body().unwrap().append_child(&input);

        // Input element gets dropped here.
        // By doing this we are verifying that the Closure was not invalidated at
        // this point since there is another Rc reference in the DomUpdater.active_closures
    }

    let input: HtmlInputElement = document
        .get_element_by_id("input-elem")
        .unwrap()
        .dyn_into()
        .unwrap();
    let input_event = InputEvent::new("input").unwrap();

    assert_eq!(&*text.borrow(), "Start Text");

    // After dispatching the oninput event our `text` should have a value of the input elements value.
    web_sys::EventTarget::from(input)
        .dispatch_event(&input_event)
        .unwrap();

    assert_eq!(&*text.borrow(), "End Text");
}
