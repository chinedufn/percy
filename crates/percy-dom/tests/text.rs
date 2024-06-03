//! Tests related to text nodes.
//!
//! To run all tests in this file:
//!
//! wasm-pack test --chrome --headless crates/percy-dom --test text

extern crate wasm_bindgen_test;
extern crate web_sys;

use wasm_bindgen_test::*;

use percy_dom::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

mod diff_patch_test_case;

use self::diff_patch_test_case::DiffPatchTest;

// TODO: Add tests that we remove the `ptns` separator comments when we remove text nodes.

/// Verify that we can replace a single text node with another text node.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test text -- replace_text_node_with_text_node
#[wasm_bindgen_test]
fn replace_text_node_with_text_node() {
    DiffPatchTest {
        desc: "Replace a text node with another text node.",
        old: html! {
         <div>
           {"Original element"}
         </div>
        },
        new: html! {
        <div>
          {"Patched element"}
        </div> },
        override_expected: None,
    }
    .test();
}

/// wasm-pack test --chrome --headless crates/percy-dom --test text -- append_text_node
#[wasm_bindgen_test]
fn append_text_node() {
    DiffPatchTest {
        desc: "Append text node",
        old: html! { <div> </div> },
        new: html! { <div> Hello </div> },
        override_expected: None,
    }
    .test();
}

/// wasm-pack test --chrome --headless crates/percy-dom --test text -- append_sibling_text_nodes
#[wasm_bindgen_test]
fn append_sibling_text_nodes() {
    let text1 = VirtualNode::text("Hello");
    let text2 = VirtualNode::text("World");

    DiffPatchTest {
        desc: "Append sibling text nodes",
        old: html! { <div> </div> },
        new: html! { <div> {text1} {text2} </div> },
        override_expected: None,
    }
    .test();
}

/// https://github.com/chinedufn/percy/issues/62
///
/// wasm-pack test --chrome --headless crates/percy-dom --test text -- replace_element_with_text_node
#[wasm_bindgen_test]
fn replace_element_with_text_node() {
    DiffPatchTest {
        desc: "#62: Replace element with text node",
        old: html! { <span> <br> </span> },
        new: html! { <span> a </span> },
        override_expected: None,
    }
    .test();
}

/// https://github.com/chinedufn/percy/issues/68
///
/// wasm-pack test --chrome --headless crates/percy-dom --test text -- text_root_node
#[wasm_bindgen_test]
fn text_root_node() {
    DiffPatchTest {
        desc: "Patching of text root node works",
        old: html! { Old text },
        new: html! { New text },
        override_expected: None,
    }
    .test();
}

/// wasm-pack test --chrome --headless crates/percy-dom --test text -- replace_text_with_element
#[wasm_bindgen_test]
fn replace_text_with_element() {
    DiffPatchTest {
        desc: "Replacing a text node with an element works",
        old: html! { <div>a</div> },
        new: html! { <div><br></div> },
        override_expected: None,
    }
    .test();
}

/// wasm-pack test --chrome --headless crates/percy-dom --test text -- text_node_siblings
#[wasm_bindgen_test]
fn text_node_siblings() {
    // TODO: Requires proc macro APIs that are currently unstable - https://github.com/rust-lang/rust/issues/54725
    // // NOTE: Since there are two text nodes next to eachother we expect a `<!--ptns-->` separator in
    // // between them.
    // // @see virtual_node/mod.rs -> create_dom_node() for more information
    // // TODO: A little more spacing than there should be in between the text nodes ... but doesn't
    // //  impact the user experience so we can look into that later..
    // let override_expected = Some(
    //     r#"<div id="after"><span> The button has been clicked:  <!--ptns--> world </span></div>"#,
    // );

    // TODO: After the proc macro span APIs stabilize remove this in favor of the above commented out
    //  code.
    //   https://github.com/rust-lang/rust/issues/54725
    let override_expected =
        Some(r#"<div id="after"><span>The button has been clicked: <!--ptns-->world</span></div>"#);

    let old1 = VirtualNode::text("The button has been clicked: ");
    let old2 = VirtualNode::text("hello");

    let new1 = VirtualNode::text("The button has been clicked: ");
    let new2 = VirtualNode::text("world");

    DiffPatchTest {
        desc: "Diff patch on text node siblings",
        old: html! {
        <div id="before">
            <span> { {old1} {old2} } </span>
        </div>
        },
        new: html! {
        <div id="after">
            <span> { {new1} {new2} } </span>
        </div>
        },
        override_expected,
    }
    .test();
}
