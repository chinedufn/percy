//! Tests that ensure that diffing and patching work properly in a real browser.
//!
//! To run all tests in this file:
//!
//! wasm-pack test --chrome --headless crates/percy-dom --test diff_patch
//!
//! TODO: Move the tests in this file to more focused files.
//!  For example, we might move the replace tests to a `replace.rs` file.

extern crate wasm_bindgen_test;
extern crate web_sys;
use wasm_bindgen_test::*;

use percy_dom::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

mod diff_patch_test_case;
use self::diff_patch_test_case::DiffPatchTest;

#[wasm_bindgen_test]
fn truncate_children() {
    DiffPatchTest {
        desc: "Truncates extra children",
        old: html! {
         <div>
           <div> <div> <b></b> <em></em> </div> </div>
         </div>
        },
        new: html! {
         <div>
           <div> <div> <b></b> </div> </div>
         </div>
        },
        override_expected: None,
    }
    .test();

    DiffPatchTest {
        desc: "https://github.com/chinedufn/percy/issues/48",
        old: html! {
         <div>
          ab <p></p> c
         </div>
        },
        new: html! {
         <div>
           ab <p></p>
         </div>
        },
        override_expected: None,
    }
    .test();
}

#[wasm_bindgen_test]
fn remove_attributes() {
    DiffPatchTest {
        desc: "Removes attributes",
        old: html! { <div style=""> </div>
        },
        new: html! { <div></div> },
        override_expected: None,
    }
    .test();
}

#[wasm_bindgen_test]
fn append_children() {
    DiffPatchTest {
        desc: "Append a child node",
        old: html! { <div> </div>
        },
        new: html! { <div> <span></span> </div> },
        override_expected: None,
    }
    .test();
}

/// wasm-pack test --chrome --headless crates/percy-dom --test diff_patch -- replace_with_children
#[wasm_bindgen_test]
fn replace_with_children() {
    DiffPatchTest {
        desc: "Replace node that has children",
        old: html! {
          <table>
            <tr>
              <th>0</th>
            </tr>
            <tr>
              <td>1</td>
            </tr>
          </table>
        },
        new: html! {
          <table>
            <tr>
              <td>2</td>
            </tr>
            <tr>
              <th>3</th>
            </tr>
          </table>
        },
        override_expected: None,
    }
    .test();
}
