//! Various helper functions and types for writing tests.

// Tests share the same DOM, so IDs need to be unique across tests.
pub fn random_id() -> &'static str {
    Box::leak(Box::new(js_sys::Math::random().to_string()))
}

pub fn document() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}

pub fn append_to_document (elem: &web_sys::Element) {
    document().body().unwrap().append_child(elem).unwrap();
}

pub fn get_element_by_id(id: &str) -> web_sys::Element {
    document().get_element_by_id(id).unwrap()
}

pub fn create_mount() -> web_sys::Element {
    let mount = document().create_element("div").unwrap();
    document().body().unwrap().append_child(&mount).unwrap();

    mount
}
