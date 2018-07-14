use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type HTMLDocument;

    pub static document: HTMLDocument;

    #[wasm_bindgen(method, js_name = createElement)]
    pub fn create_element(this: &HTMLDocument, tag: &str) -> Element;

    #[wasm_bindgen(method, js_name = createElement)]
    pub fn create_canvas_element(this: &HTMLDocument, tag: &str) -> HTMLCanvasElement;

    #[wasm_bindgen(method, getter)]
    pub fn body(this: &HTMLDocument) -> Element;
}

#[wasm_bindgen]
extern "C" {
    pub type Element;

    #[wasm_bindgen(method, js_name = appendChild)]
    pub fn append_child(this: &Element, other: Element);

    pub type HTMLCanvasElement;
}