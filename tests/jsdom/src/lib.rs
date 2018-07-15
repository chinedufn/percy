#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

#[macro_use]
extern crate virtual_dom_rs;
use virtual_dom_rs::webapis::*;

#[wasm_bindgen]
pub fn nested_divs () -> Element {
    let div = html! { <div> <div> <div></div> </div> </div> };
    div.create_element()
}

#[wasm_bindgen]
pub fn div_with_properties () -> Element {
    let div = html! { <div id="id-here", class="two classes",></div> };
    div.create_element()
}

