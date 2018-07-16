#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate isomorphic_app;
use isomorphic_app::App;
use isomorphic_app::Element;

#[wasm_bindgen]
pub struct Client {
    app: App,
    root_node: Element
}

#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new(initial_state: &str, root_node: Element) -> Client {
        Client {
            app: App::from_state_json(initial_state),
            root_node
        }
    }

    pub fn render(&self) -> Element {
        self.app.render().create_element()

//        let mut old_elem = html! { <div id="old",> { "Original element" } </div> };
//
//        let root_node = old_elem.create_element();
//        document.body().append_child(root_node);
//        let root_node = document.get_element_by_id("old");
//
//        let mut new_elem = html! { <div id="patched",> { "Patched element" } </div> };
//
//        let patches = virtual_dom_rs::diff(&old_elem, &mut new_elem);
//
//        virtual_dom_rs::patch(&root_node, &patches);
    }
}
