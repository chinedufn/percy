#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

#[macro_use]
extern crate virtual_dom_rs;
use std::cell::Cell;
use std::rc::Rc;
use virtual_dom_rs::virtual_node::VirtualNode;
use virtual_dom_rs::webapis::*;

macro_rules! clog {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

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
             <div id="old",>
               <div> <div> <b></b> </div> </div>
             </div>
            },
            desc: "Truncates extra children",
        })
    }
}

fn test_patch(test_case: PatchTestCase) {
    let root_node = test_case.old.create_element();

    document.body().append_child(&root_node);

    let patches = virtual_dom_rs::diff(&test_case.old, &test_case.new);
//    clog!("{:#?}", patches);

    virtual_dom_rs::patch(root_node.clone(), &test_case.old, &patches);

    let new_root_node_id = test_case.new.props.get("id").unwrap();

    let new_root_node = document.get_element_by_id(new_root_node_id);
    let new_root_node = new_root_node.outer_html();

    let expected_new_root_node = test_case.new.to_string();

    if new_root_node == expected_new_root_node {
        clog!("PASSED {}", test_case.desc);
    } else {
        clog!(
            "\nFailed diff/patch operation\nActual: {}\nExpected: {}\nMessage: {}\n",
            new_root_node,
            expected_new_root_node,
            test_case.desc
        );
        panic!("Failure");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {}
}
