//! Tests related to replacing nodes.
//!
//! To run all tests in this file:
//!
//! wasm-pack test --chrome --headless crates/percy-dom --test replace

extern crate wasm_bindgen_test;
extern crate web_sys;
use wasm_bindgen_test::*;

use percy_dom::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

mod diff_patch_test_case;
use self::diff_patch_test_case::DiffPatchTest;

/// Verify that we can replace the first sibling in a list of siblings.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test replace -- replace_first_sibling_node
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
/// wasm-pack test --chrome --headless crates/percy-dom --test replace -- replace_first_sibling_text
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
