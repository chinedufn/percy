#![feature(use_extern_macros)]

extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type HTMLDocument;

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(msg: &str);

    pub static document: HTMLDocument;

    #[wasm_bindgen(method, js_name = createElement)]
    pub fn create_element(this: &HTMLDocument, tag: &str) -> Element;

    #[wasm_bindgen(method, js_name = getElementById)]
    pub fn get_element_by_id(this: &HTMLDocument, id: &str) -> Element;

    #[wasm_bindgen(method, js_name = createElement)]
    pub fn create_canvas_element(this: &HTMLDocument, tag: &str) -> HTMLCanvasElement;

    #[wasm_bindgen(method, js_name = createTextNode)]
    pub fn create_text_node(this: &HTMLDocument, text: &str) -> Text;

    #[wasm_bindgen(method, getter)]
    pub fn body(this: &HTMLDocument) -> Element;
}

impl Clone for Element {
    fn clone(&self) -> Element {
        Element {
            obj: self.obj.clone(),
        }
    }
}

#[wasm_bindgen]
extern "C" {
    pub type Element;

    #[wasm_bindgen(method, js_name = appendChild)]
    pub fn append_child(this: &Element, other: &Element);

    #[wasm_bindgen(method, js_name = appendChild)]
    pub fn append_text_child(this: &Element, other: Text);

    #[wasm_bindgen(method, js_name = setAttribute)]
    pub fn set_attribute(this: &Element, attrib: &str, value: &str);

    #[wasm_bindgen(method, js_name = removeAttribute)]
    pub fn remove_attribute(this: &Element, attrib: &str);

    #[wasm_bindgen(method, js_name = addEventListener)]
    pub fn add_event_listener(this: &Element, event: &str, cb: &Closure<Fn()>);

    #[wasm_bindgen(method, getter, js_name = parentElement)]
    pub fn parent_element(this: &Element) -> Element;

    #[wasm_bindgen(method, js_name = replaceChild)]
    pub fn replace_child(this: &Element, new_child: &Element, old_child: &Element);

    #[wasm_bindgen(method, js_name = removeChild)]
    pub fn remove_child(this: &Element, remove: &Element);

    #[wasm_bindgen(method, getter, js_name = lastChild)]
    pub fn last_child(this: &Element) -> Element;

    #[wasm_bindgen(method, getter, js_name = childNodes)]
    pub fn child_nodes(this: &Element) -> NodeList;

    #[wasm_bindgen(method, js_name = replaceWith)]
    pub fn replace_with(this: &Element, replace_with: &Element);

    #[wasm_bindgen(method, getter, js_name = outerHTML)]
    pub fn outer_html(this: &Element) -> String;

    #[wasm_bindgen(method, setter = nodeValue)]
    pub fn set_node_value(this: &Element, value: &str);

    pub type HTMLCanvasElement;
}

#[wasm_bindgen]
extern "C" {
    pub type NodeList;

    #[wasm_bindgen(method)]
    pub fn item(this: &NodeList, idx: usize) -> Element;

    #[wasm_bindgen(method, getter)]
    pub fn length(this: &NodeList) -> u32;
}

#[wasm_bindgen]
extern "C" {
    pub type Text;
}
