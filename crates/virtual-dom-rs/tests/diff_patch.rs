extern crate wasm_bindgen_test;
extern crate web_sys;
use wasm_bindgen_test::*;

use virtual_dom_rs::virtual_node::VirtualNode;
use web_sys::*;

#[macro_use]
extern crate virtual_dom_rs;

wasm_bindgen_test_configure!(run_in_browser);

struct DiffPatchTest {
    old: VirtualNode,
    new: VirtualNode,
    desc: &'static str,
    override_expected: Option<String>,
}

#[wasm_bindgen_test]
fn replace_child() {
    DiffPatchTest {
        old: html! {
         <div id="old",>
           { "Original element" }
         </div>
        },
        new: html! { <div id="patched",> { "Patched element" }</div> },
        desc: "Replace a root node attribute attribute and a child text node",
        override_expected: None,
    }
    .test();
}

#[wasm_bindgen_test]
fn truncate_children() {
    DiffPatchTest {
        old: html! {
         <div id="old",>
           <div> <div> <b></b> <em></em> </div> </div>
         </div>
        },
        new: html! {
         <div id="new",>
           <div> <div> <b></b> </div> </div>
         </div>
        },
        desc: "Truncates extra children",
        override_expected: None,
    }
    .test();
}

#[wasm_bindgen_test]
fn remove_attributes() {
    DiffPatchTest {
        old: html! { <div id="remove-attrib", style="",> </div>
        },
        new: html! { <div id="new-root",></div> },
        desc: "Removes attributes",
        override_expected: None,
    }
    .test();
}

#[wasm_bindgen_test]
fn append_children() {
    DiffPatchTest {
        desc: "Append a child node",
        old: html! { <div id="foo",> </div>
        },
        new: html! { <div id="bar",> <span></span> </div> },
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

    DiffPatchTest {
        desc: "Diff patch on text node siblings",
        old: html! {
        <div id="before",>
            <span> { "The button has been clicked: "  "hello"} </span>
        </div>
        },
        new: html! {
        <div id="after",>
            <span> { "The button has been clicked: "  "world"} </span>
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
        old: html! { <div id="foo",> </div> },
        new: html! { <div id="foo",> {"Hello"} </div> },
        override_expected: None,
    }
    .test();
}

#[wasm_bindgen_test]
fn append_sibling_text_nodes() {
    DiffPatchTest {
        desc: "Append sibling text nodes",
        old: html! { <div id="bar",> </div> },
        new: html! { <div id="bang",> {"Hello"} {"World"} </div> },
        override_expected: None,
    }
    .test();
}

impl DiffPatchTest {
    fn test(&self) {
        let document = web_sys::window().unwrap().document().unwrap();
        let root_node = self.old.create_element();

        (document.body().unwrap().as_ref() as &web_sys::Node)
            .append_child(&root_node.as_ref() as &web_sys::Node)
            .unwrap();

        let patches = virtual_dom_rs::diff(&self.old, &self.new);

        virtual_dom_rs::patch(root_node, &patches);

        let new_root_node_id = self.new.props.get("id").unwrap();

        let new_root_node = document.get_element_by_id(new_root_node_id).unwrap();
        let new_root_node = new_root_node.outer_html();

        let expected_new_root_node = match self.override_expected {
            Some(ref expected) => expected.clone(),
            None => self.new.to_string(),
        };

        assert_eq!(new_root_node, expected_new_root_node, "{}", self.desc);
    }
}
