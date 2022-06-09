use std::sync::Arc;
use wasm_bindgen::prelude::*;

use crate::async_task_spawner::WebAsyncTaskSpawner;
use percy_preview::{Preview, Rerender};
use percy_preview_app::{PercyPreviewWebClient, WebClientConfig};

/// All of the view components to preview.
fn previews(rerender: Rerender) -> Vec<Preview> {
    use crate::views;

    vec![
        views::login::preview::login_form_preview(rerender.clone()),
        views::side_nav::preview::side_nav_preview(rerender.clone()),
    ]
}

/// Start the Percy preview application.
#[wasm_bindgen]
pub fn start_component_preview(dom_selector_of_mount: &str) -> PercyPreviewWebClient {
    let rerender = Rerender::new();

    let rerender_clone = rerender.clone();

    let previews = previews(rerender);

    let client = PercyPreviewWebClient::new_append_to_mount(
        WebClientConfig {
            async_task_spawner: Arc::new(WebAsyncTaskSpawner),
            previews,
        },
        dom_selector_of_mount,
    );

    let rerender = client.rerender.clone();
    rerender_clone.set_render_fn(Box::new(move || (rerender.lock().unwrap())()));

    client
}
