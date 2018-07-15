#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

#[macro_use]
extern crate virtual_dom_rs;
use virtual_dom_rs::webapis::*;
use std::cell::Cell;
use std::rc::Rc;

#[wasm_bindgen]
pub fn nested_divs () -> Element {
    let mut div = html! { <div> <div> <div></div> </div> </div> };
    div.create_element()
}

#[wasm_bindgen]
pub fn div_with_properties () -> Element {
    let mut div = html! { <div id="id-here", class="two classes",></div> };
    div.create_element()
}

#[wasm_bindgen]
pub struct ClickTest {
    clicked: Rc<Cell<bool>>
}

#[wasm_bindgen]
impl ClickTest {
    #[wasm_bindgen(constructor)]
    pub fn new () -> ClickTest {
        ClickTest { clicked: Rc::new(Cell::new(false))}
    }

    pub fn get_clicked(&self) -> bool { self.clicked.get() }

    pub fn div_with_click_event (&self) -> Element {
        let mut clicked = Rc::clone(&self.clicked);

        let mut div = html! { <div
         !onclick=move || {
           clicked.set(true);
         },
        >
        </div> };

        div.create_element()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works () {

    }
}
