use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

use percy_preview::{Preview, Rerender};
use percy_preview_app::{AppConfig, PercyPreviewWebClient};

/// All of the view components to preview.
fn previews(rerender: Rerender) -> Vec<Preview> {
    use crate::views;

    vec![views::controls_header::preview::controls_header_preview(
        rerender,
    )]
}

/// Start the Percy preview application.
#[wasm_bindgen]
pub fn start_component_preview(dom_selector_of_mount: &str) {
    let rerender = Rerender::new(Arc::new(Mutex::new(Box::new(move || {}))));

    let rerender_clone = rerender.clone();

    let previews = previews(rerender);

    let client =
        PercyPreviewWebClient::new_append_to_mount(AppConfig { previews }, dom_selector_of_mount);

    rerender_clone.set_render_fn(Box::new(move || (client.rerender.lock().unwrap())()));
}
