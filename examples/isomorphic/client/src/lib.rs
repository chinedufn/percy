#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate isomorphic_app;
use isomorphic_app::App;
use isomorphic_app::Element;

#[wasm_bindgen]
pub struct Client {
    app: App
}

#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new(initial_state: &str) -> Client {
        Client {
            app: App::from_state_json(initial_state)
        }
    }

    pub fn render(&self) -> Element {
        self.app.render().create_element()
    }
}
