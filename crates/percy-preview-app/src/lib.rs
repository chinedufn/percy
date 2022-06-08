//! Used to run an web application that lets you preview view components.

#![deny(missing_docs)]

#[macro_use]
extern crate sunbeam;

pub use self::app::AppConfig;
use self::app::PercyPreviewApp;
use self::render::render_app;
use std::sync::{Arc, Mutex};

use percy_dom::{render::create_render_scheduler, PercyDom, VirtualNode};
use routes::create_router;
use wasm_bindgen::prelude::*;

mod app;
mod render;
mod routes;
mod view;

pub mod all_sunbeam_css;

/// A frontend web application that lets you preview your own application's view components.
#[wasm_bindgen]
pub struct PercyPreviewWebClient {
    /// Rerender the application.
    #[wasm_bindgen(skip)]
    pub rerender: Arc<Mutex<Box<dyn FnMut() -> ()>>>,
}

impl PercyPreviewWebClient {
    /// Create a new preview application and append the application to a DOM node.
    pub fn new_append_to_mount(config: AppConfig, dom_selector_of_mount: &str) -> Self {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        let render: Arc<Mutex<Box<dyn FnMut() -> ()>>> = Arc::new(Mutex::new(Box::new(|| {})));
        let render_clone = Arc::clone(&render);

        let app = PercyPreviewApp::new(config, render);

        let pdom = create_percy_dom(dom_selector_of_mount, render_app(&app.world));

        let render = move || render_app(&app.world);
        let render = create_render_scheduler(pdom, render);

        *render_clone.lock().unwrap() = render;

        Self {
            rerender: render_clone,
        }
    }
}

fn create_percy_dom(dom_selector_of_mount: &str, start_view: VirtualNode) -> PercyDom {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let mount = document
        .query_selector(dom_selector_of_mount)
        .unwrap()
        .unwrap();
    let pdom = PercyDom::new_append_to_mount(start_view, &mount);
    pdom
}
