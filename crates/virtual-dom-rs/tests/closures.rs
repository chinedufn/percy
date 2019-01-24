//! Ensure that our DomUpdater maintains Rc's to closures so that they work even
//! after dropping virtual dom nodes.

use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::rc::Rc;
use virtual_dom_rs::html;
use virtual_dom_rs::prelude::*;
use virtual_dom_rs::recurse_html;
use virtual_dom_rs::DomUpdater;
use wasm_bindgen::JsCast;
use wasm_bindgen_test;
use wasm_bindgen_test::*;
use web_sys::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn closure_not_dropped() {
    let text = Rc::new(RefCell::new("Start Text".to_string()));

    let document = web_sys::window().unwrap().document().unwrap();

    let mut dom_updater = None;

    {
        let mut input = make_input_component(Rc::clone(&text));
        input.props.insert("id".into(), "old-input-elem".into());

        let mount = document.create_element("div").unwrap();
        mount.set_id("mount");
        document.body().unwrap().append_child(&mount).unwrap();

        dom_updater = Some(DomUpdater::new_replace_mount(input, mount));

        let mut dom_updater = dom_updater.as_mut().unwrap();

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
        new_node.props.insert("id".into(), "new-input-elem".into());

        dom_updater.update(new_node);
    }

    web_sys::console::log_1(&format!("{}", document.body().unwrap().inner_html()).into());

    let dom_updater = dom_updater.unwrap();

    web_sys::console::log_1(
        &format!(
            "Closures {}",
            dom_updater.active_closures.get(&1).unwrap().len()
        )
        .into(),
    );

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
}

fn make_input_component(text_clone: Rc<RefCell<String>>) -> VirtualNode {
    html! {
        <input
           // On input we'll set our Rc<RefCell<String>> value to the input elements value
           !oninput=move |event: Event| {
              web_sys::console::log_1(&format!("RAN!@!(@!!!@").into());
              let input_elem = event.target().unwrap();
              let input_elem = input_elem.dyn_into::<HtmlInputElement>().unwrap();
              *text_clone.borrow_mut() = input_elem.value();
           },
           value="End Text",
        >
        </input>
    }
}
