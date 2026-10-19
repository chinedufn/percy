//! Tests related to replacing nodes.
//!
//! To run all tests in this file:
//!
//! CHROMEDRIVER=chromedriver cargo test -p percy-dom --target wasm32-unknown-unknown --test replace

extern crate wasm_bindgen_test;
extern crate web_sys;
use wasm_bindgen_test::*;

use percy_dom::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

mod diff_patch_test_case;
use self::diff_patch_test_case::DiffPatchTest;

/// Verify that we can replace the first sibling in a list of siblings.
///
/// CHROMEDRIVER=chromedriver cargo test -p percy-dom --target wasm32-unknown-unknown --test replace -- replace_first_sibling_node
#[wasm_bindgen_test]
fn replace_first_sibling_node() {
    DiffPatchTest {
        desc: "Replace first sibling node.",
        old: html! {
         <div>
            <div></div>
            <em></em>
         </div>
        },
        new: html! {
        <div>
            <span></span>
            <em></em>
        </div>
        },
        override_expected: None,
    }
    .test();
}

/// Verify that we can replace the first sibling text in a list of siblings.
///
/// CHROMEDRIVER=chromedriver cargo test -p percy-dom --target wasm32-unknown-unknown --test replace -- replace_first_sibling_text
#[wasm_bindgen_test]
fn replace_first_sibling_text() {
    DiffPatchTest {
        desc: "Replace first sibling text.",
        old: html! {
         <div>
            {"Some text"}
            <em></em>
         </div>
        },
        new: html! {
        <div>
            <span></span>
            <em></em>
        </div> },
        override_expected: None,
    }
    .test();
}

/// Verify that after the root node is replaced because its key changed, later updates are applied
/// to the new root node that is in the DOM, instead of to the old root node that was replaced.
///
/// CHROMEDRIVER=chromedriver cargo test -p percy-dom --target wasm32-unknown-unknown --test replace -- updates_after_root_replaced_by_key_change
#[wasm_bindgen_test]
fn updates_after_root_replaced_by_key_change() {
    let html = mount_then_update(vec![
        html! { <div key="a"><span></span></div> },
        html! { <div key="b"><span></span></div> },
        html! { <div key="b"><span><em></em></span></div> },
    ]);

    assert_eq!(html, "<div><span><em></em></span></div>");
}

/// Verify that after the root node is replaced because its tag changed, later updates are applied
/// to the new root node that is in the DOM, instead of to the old root node that was replaced.
///
/// CHROMEDRIVER=chromedriver cargo test -p percy-dom --target wasm32-unknown-unknown --test replace -- updates_after_root_replaced_by_tag_change
#[wasm_bindgen_test]
fn updates_after_root_replaced_by_tag_change() {
    let html = mount_then_update(vec![
        html! { <div><span></span></div> },
        html! { <section><span></span></section> },
        html! { <section><span><em></em></span></section> },
    ]);

    assert_eq!(html, "<section><span><em></em></span></section>");
}

/// Verify that we can update the DOM after replacing the root node with a root node that has fewer
/// children.
///
/// CHROMEDRIVER=chromedriver cargo test -p percy-dom --target wasm32-unknown-unknown --test replace -- update_after_root_replaced_with_fewer_children
#[wasm_bindgen_test]
fn update_after_root_replaced_with_fewer_children() {
    let html = mount_then_update(vec![
        html! { <div key="app"><nav></nav><main><p></p></main></div> },
        html! { <div>{"Loading..."}</div> },
        html! { <div key="app"><nav></nav><main><p></p></main></div> },
    ]);

    assert_eq!(html, "<div><nav></nav><main><p></p></main></div>");
}

/// Mount the first virtual node, update to each of the following virtual nodes, then return the
/// mount's inner HTML.
fn mount_then_update(mut vdoms: Vec<VirtualNode<web_sys::Window>>) -> String {
    let mount = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("div")
        .unwrap();

    let mut pdom = PercyDom::new_append_to_mount(vdoms.remove(0), &mount);
    for vdom in vdoms {
        pdom.update(vdom);
    }

    mount.inner_html()
}
