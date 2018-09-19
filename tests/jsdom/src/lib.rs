/// TODO: Migrate this file to use wasm-bindgen-test and test in chrome and firefox

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate web_sys;
use web_sys::console;

extern crate js_sys;

#[macro_use]
extern crate virtual_dom_rs;
use std::cell::Cell;
use std::rc::Rc;
use virtual_dom_rs::virtual_node::VirtualNode;
use virtual_dom_rs::web_sys::*;

#[wasm_bindgen]
pub fn nested_divs() -> Element {
    let mut div = html! { <div> <div> <div></div> </div> </div> };
    div.create_element()
}

#[wasm_bindgen]
pub fn div_with_properties() -> Element {
    let mut div = html! { <div id="id-here", class="two classes",></div> };
    div.create_element()
}

#[wasm_bindgen]
pub struct ClickTest {
    clicked: Rc<Cell<bool>>,
}

#[wasm_bindgen]
impl ClickTest {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ClickTest {
        ClickTest {
            clicked: Rc::new(Cell::new(false)),
        }
    }

    pub fn get_clicked(&self) -> bool {
        self.clicked.get()
    }

    pub fn div_with_click_event(&self) -> Element {
        let clicked = Rc::clone(&self.clicked);

        let div = html! { <div
         !onclick=move || {
           clicked.set(true);
         },
        >
        </div> };

        div.create_element()
    }
}

#[wasm_bindgen]
pub struct PatchTest {}

struct PatchTestCase {
    old: VirtualNode,
    new: VirtualNode,
    desc: &'static str,
}

#[wasm_bindgen]
impl PatchTest {
    #[wasm_bindgen(constructor)]
    pub fn new() -> PatchTest {
        PatchTest {}
    }

    pub fn run_tests(&self) {
        self.replace_child();
        self.truncate_children();
        self.remove_attributes();
        self.append_children();
        self.text_node_siblings();
        self.append_text_node();
        self.append_sibling_text_nodes();
    }
}

impl PatchTest {
    fn replace_child(&self) {
        test_patch(PatchTestCase {
            old: html! {
             <div id="old",>
               { "Original element" }
             </div>
            },
            new: html! { <div id="patched",> { "Patched element" }</div> },
            desc: "Replace a root node attribute attribute and a child text node",
        })
    }

    fn truncate_children(&self) {
        test_patch(PatchTestCase {
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
        })
    }

    fn remove_attributes(&self) {
        test_patch(PatchTestCase {
            old: html! { <div id="remove-attrib", style="",> </div>
            },
            new: html! { <div id="new-root",></div> },
            desc: "Removes attributes",
        })
    }

    fn append_children(&self) {
        test_patch(PatchTestCase {
            old: html! { <div id="foo",> </div>
            },
            new: html! { <div id="bar",> <span></span> </div> },
            desc: "Append a child node",
        })
    }

    fn text_node_siblings(&self) {
        let old = html! {
        <div id="before",>
            <span> { "The button has been clicked: "  "hello"} </span>
        </div>};

        let new = html! {
        <div id="after",>
            <span> { "The button has been clicked: "  "world"} </span>
        </div>};

        let document = web_sys::window().unwrap().document().unwrap();
        let root_node = old.create_element();

        (document.body().unwrap().as_ref() as &web_sys::Node)
            .append_child(&root_node.as_ref() as &web_sys::Node)
            .unwrap();

        let patches = virtual_dom_rs::diff(&old, &new);
        let log = &format!("{:#?}", patches);
        clog(log);

        virtual_dom_rs::patch(root_node, &patches);

        // TODO: Print an error if the new test case doesn't have an id set...
        let new_root_node_id = new.props.get("id").unwrap();

        let new_root_node = document.get_element_by_id(new_root_node_id).unwrap();
        let new_root_node = new_root_node.outer_html();

        // NOTE: Since there are two text nodes next to eachother we expect a `<!--ptns-->` separator in
        // between them.
        // @see virtual_node/mod.rs -> create_element() for more information
        let expected_new_root_node =
            r#"<div id="after"><span>The button has been clicked: <!--ptns-->world</span></div>"#;

        if new_root_node == expected_new_root_node {
            let log = &format!("PASSED {}", "Diff patch on text node siblings");
            clog(log);
        } else {
            let log = &format!(
                "\nFailed diff/patch operation\nActual: {}\nExpected: {}\nMessage: {}\n",
                new_root_node, expected_new_root_node, "Diff match on text node siblings"
            );
            clog(log);
            panic!("Failure");
        }
    }

    fn append_text_node(&self) {
        test_patch(PatchTestCase {
            old: html! { <div id="foo",> </div> },
            new: html! { <div id="foo",> {"Hello"} </div> },
            desc: "Append text node",
        })
    }

    fn append_sibling_text_nodes(&self) {
        test_patch(PatchTestCase {
            old: html! { <div id="bar",> </div> },
            new: html! { <div id="bang",> {"Hello"} {"World"} </div> },
            desc: "Append sibling text nodes",
        })
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn clog(s: &str);
}

fn test_patch(test_case: PatchTestCase) {
    let document = web_sys::window().unwrap().document().unwrap();
    let root_node = test_case.old.create_element();

    (document.body().unwrap().as_ref() as &web_sys::Node)
        .append_child(&root_node.as_ref() as &web_sys::Node)
        .unwrap();

    let patches = virtual_dom_rs::diff(&test_case.old, &test_case.new);
    let log = &format!("{:#?}", patches);
    clog(log);

    virtual_dom_rs::patch(root_node, &patches);

    // TODO: Print an error if the new test case doesn't have an id set...
    let new_root_node_id = test_case.new.props.get("id").unwrap();

    let new_root_node = document.get_element_by_id(new_root_node_id).unwrap();
    let new_root_node = new_root_node.outer_html();

    let expected_new_root_node = test_case.new.to_string();

    if new_root_node == expected_new_root_node {
        let log = &format!("PASSED {}", test_case.desc);
        clog(log);
    } else {
        let log = &format!(
            "\nFailed diff/patch operation\nActual: {}\nExpected: {}\nMessage: {}\n",
            new_root_node, expected_new_root_node, test_case.desc
        );
        clog(log);
        panic!("Failure");
    }
}
