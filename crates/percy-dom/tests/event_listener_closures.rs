//! Ensure that our DomUpdater maintains Rc's to closures so that they work even
//! after dropping virtual dom nodes.
//!
//! To run all tests in this file:
//!
//! wasm-pack test --chrome --headless crates/percy-dom --test closures

use percy_dom::prelude::*;
use percy_dom::DomUpdater;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen_test;
use wasm_bindgen_test::*;
use web_sys::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Verify that if we create a real DOM element from a virtual node that has an event listener and
/// then drop the virtual node, the event listener still works.
/// This ensures that the DomUpdater is holding onto a reference counted pointer to the event
/// listener closure.
#[wasm_bindgen_test]
fn closure_not_dropped() {
    let text = Rc::new(RefCell::new("Start Text".to_string()));

    let document = web_sys::window().unwrap().document().unwrap();

    let mut dom_updater;

    {
        let mut input = make_input_component(Rc::clone(&text));
        input
            .as_velement_mut()
            .expect("Not an element")
            .attrs
            .insert("id".into(), "old-input-elem".into());

        let mount = document.create_element("div").unwrap();
        mount.set_id("mount");
        document.body().unwrap().append_child(&mount).unwrap();

        dom_updater = Some(DomUpdater::new_replace_mount(input, mount));

        let dom_updater = dom_updater.as_mut().unwrap();

        // Input VirtualNode from above gets dropped at the end of this block,
        // yet that element held Rc's to the Closure's that power the oninput event.
        //
        // We're patching the DOM with a new vdom, but since our new vdom doesn't contain any
        // new elements, `.create_element` won't get called and so no new Closures will be
        // created.
        //
        // So, we're testing that our old Closure's still work. The reason that they work is
        // that dom_updater maintains Rc's to those Closures.
        let mut new_node = make_input_component(Rc::clone(&text));
        new_node
            .as_velement_mut()
            .expect("Not an element")
            .attrs
            .insert("id".into(), "new-input-elem".into());

        dom_updater.update(new_node);
    }

    let dom_updater = dom_updater.as_ref().unwrap();

    let input: HtmlInputElement = document
        .get_element_by_id("new-input-elem")
        .expect("Input element")
        .dyn_into()
        .unwrap();
    let input_event = InputEvent::new("input").unwrap();

    assert_eq!(&*text.borrow(), "Start Text");

    // After dispatching the oninput event our `text` should have a value of the input elements value.
    web_sys::EventTarget::from(input)
        .dispatch_event(&input_event)
        .unwrap();

    assert_eq!(&*text.borrow(), "End Text");

    assert_eq!(
        dom_updater.active_closures.get(&1).as_ref().unwrap().len(),
        1
    );
}

// We're just making sure that things compile - other tests give us confidence that the closure
// will work just fine.
//
// https://github.com/chinedufn/percy/issues/81
//
//#[wasm_bindgen_test]
//fn closure_with_no_params_compiles() {
//    let _making_sure_this_works = html! {
//        <div onclick=|| {}></div>
//    };
//}

fn make_input_component(text_clone: Rc<RefCell<String>>) -> VirtualNode {
    html! {
        <input
           // On input we'll set our Rc<RefCell<String>> value to the input elements value
           oninput=move |event: InputEvent| {
              let input_elem = event.target().unwrap();
              let input_elem = input_elem.dyn_into::<HtmlInputElement>().unwrap();
              *text_clone.borrow_mut() = input_elem.value();
           }
           value="End Text"
        >
    }
}
