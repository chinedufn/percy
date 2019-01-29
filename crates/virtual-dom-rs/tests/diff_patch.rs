#![feature(proc_macro_hygiene)]

extern crate wasm_bindgen_test;
extern crate web_sys;
use wasm_bindgen_test::*;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use virtual_dom_rs::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

struct DiffPatchTest {
    desc: &'static str,
    old: VirtualNode,
    new: VirtualNode,
    override_expected: Option<String>,
}

#[wasm_bindgen_test]
fn replace_child() {
    DiffPatchTest {
        desc: "Replace a root node attribute attribute and a child text node",
        old: html! {
         <div>
           Original element
         </div>
        },
        new: html! { <div> Patched element</div> },
        override_expected: None,
    }
    .test();
}

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

#[wasm_bindgen_test]
fn text_node_siblings() {
    // NOTE: Since there are two text nodes next to eachother we expect a `<!--ptns-->` separator in
    // between them.
    // @see virtual_node/mod.rs -> create_element() for more information
    let override_expected = Some(
        r#"<div id="after"><span>The button has been clicked: <!--ptns-->world</span></div>"#
            .to_string(),
    );

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

#[wasm_bindgen_test]
fn replace_with_children() {
    DiffPatchTest {
        desc: "Replace node that has children",
        old: html! { <table><tr><th>0</th></tr><tr><td>1</td></tr></table> },
        new: html! { <table><tr><td>2</td></tr><tr><th>3</th></tr></table> },
        override_expected: None,
    }
    .test();
}

// https://github.com/chinedufn/percy/issues/62
#[wasm_bindgen_test]
fn issue_62() {
    DiffPatchTest {
        desc: "Fix issue #62",
        old: html! { <span><br></span> },
        new: html! { <span>a<br></span> },
        override_expected: None,
    }
        .test();
}


impl DiffPatchTest {
    fn test(&mut self) {
        let document = web_sys::window().unwrap().document().unwrap();

        // If we haven't set an id for our element we hash the description of the test and set
        // that as the ID.
        // We need an ID in order to find the element within the DOM, otherwise we couldn't run
        // our assertions.
        if self.old.props.get("id").is_none() {
            let mut hashed_desc = DefaultHasher::new();

            self.desc.hash(&mut hashed_desc);

            self.old
                .props
                .insert("id".to_string(), hashed_desc.finish().to_string());
        }

        // Add our old node into the DOM
        let root_node = self.old.create_element().element;
        document.body().unwrap().append_child(&root_node).unwrap();

        let elem_id = self.old.props.get("id").unwrap().clone();
        // This is our root node that we're about to patch.
        // It isn't actually patched yet.. but by the time we use this it will be.
        let patched_element = document.get_element_by_id(&elem_id).unwrap();

        let patches = virtual_dom_rs::diff(&self.old, &self.new);

        virtual_dom_rs::patch(root_node, &patches);

        let expected_new_root_node = match self.override_expected {
            Some(ref expected) => expected.clone(),
            None => self.new.to_string(),
        };

        assert_eq!(
            patched_element.outer_html(),
            expected_new_root_node,
            "{}",
            self.desc
        );
    }
}
