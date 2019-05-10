use wasm_bindgen::JsValue;

pub enum Msg {
    Click,
    SetPath(String),
    /// Deserializes JSON array of Github contributors to `Option<Vec<PercyContributor>>`
    SetContributorsJson(JsValue),
    /// Represents whether the client is already fetching the JSON array of Github contributors
    InitiatedContributorsDownload,
}
