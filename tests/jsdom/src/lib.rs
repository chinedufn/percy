#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

#[macro_use]
extern crate virtual_dom_rs;
use std::cell::Cell;
use std::rc::Rc;
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

        let mut div = html! { <div
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

#[wasm_bindgen]
impl PatchTest {
    #[wasm_bindgen(constructor)]
    pub fn new() -> PatchTest {
        PatchTest {}
    }

    pub fn patch_element(&self) {
        let mut old_elem = html! { <div id="old",> { "Original element" } </div> };

        let root_node = old_elem.create_element();
        document.body().append_child(root_node);
        let root_node = document.get_element_by_id("old");

        let mut new_elem = html! { <div id="patched",> { "Patched element" } </div> };

        let mut patches = virtual_dom_rs::diff(&old_elem, &new_elem);
        clog!("{:#?}", patches);

        virtual_dom_rs::patch(&root_node, &patches);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {}
}
