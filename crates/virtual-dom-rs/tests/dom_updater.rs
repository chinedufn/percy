//! Ensure that our DomUpdater maintains Rc's to closures so that they work even
//! after dropping virtual dom nodes.

use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use virtual_dom_rs::prelude::*;
use virtual_dom_rs::DomUpdater;
use wasm_bindgen::JsCast;
use wasm_bindgen_test;
use wasm_bindgen_test::*;
use web_sys::*;

wasm_bindgen_test_configure!(run_in_browser);

// Verify that our DomUpdater's patch method works.
// We test a simple case here, since diff_patch.rs is responsible for testing more complex
// diffing and patching.
#[wasm_bindgen_test]
fn patches_dom() {
    let document = web_sys::window().unwrap().document().unwrap();

    let vdom = html! { <div></div> };
    let mut dom_updater = DomUpdater::new(vdom);


    let new_vdom = html! { <div id="patched"></div> };
    dom_updater.update(new_vdom);

    document.body().unwrap().append_child(&dom_updater.root_node());
    assert_eq!(document.query_selector("#patched").unwrap().is_some(), true);
}
