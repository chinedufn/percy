#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate isomorphic_app;
use isomorphic_app::App;

#[wasm_bindgen]
pub struct Client {}

#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Client {
        Client {}
    }
}
