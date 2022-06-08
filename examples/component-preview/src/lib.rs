//! Used to run the photomules-app in a browser.

#![deny(missing_docs)]

#[macro_use]
extern crate sunbeam;

use percy_dom::render::create_render_scheduler;
use percy_dom::{PercyDom, VElement, VirtualNode};
use std::sync::{Arc, Mutex};

use crate::routes::render_active_route;
use wasm_bindgen::prelude::*;

#[cfg(feature = "preview")]
mod preview;
#[cfg(feature = "preview")]
pub use self::preview::start_component_preview;
mod routes;
mod views;

/// Photomules' dashboard web client
#[wasm_bindgen]
pub struct PhotomulesWebClient;

#[wasm_bindgen]
impl PhotomulesWebClient {
    /// Create a new instance of the web client application.
    #[wasm_bindgen(constructor)]
    pub fn new(dom_selector_of_mount: &str) -> PhotomulesWebClient {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        let mut pdom = create_percy_dom(dom_selector_of_mount);
        pdom.update(render_active_route());

        let render = Arc::new(Mutex::new(Box::new(|| {}) as Box<dyn FnMut() -> ()>));
        let render_clone = Arc::clone(&render);

        let render = move || render_active_route();
        let render = create_render_scheduler(pdom, render);

        *render_clone.lock().unwrap() = render;

        PhotomulesWebClient
    }
}

fn create_percy_dom(dom_selector_of_mount: &str) -> PercyDom {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let mount = document
        .query_selector(dom_selector_of_mount)
        .unwrap()
        .unwrap();

    let start_view = VirtualNode::Element(VElement::new("div"));

    let dom_updater = PercyDom::new_append_to_mount(start_view, &mount);

    dom_updater
}
