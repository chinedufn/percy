use wasm_bindgen::JsValue;

pub enum Msg {
    Click,
    SetPath(String),
    StoreContributors(JsValue),
}
