//! Used to run an web application that lets you preview view components.

#![deny(missing_docs)]

#[macro_use]
extern crate sunbeam;

use self::app::PercyPreviewApp;
use self::render::render_app;
use std::sync::{Arc, Mutex};

pub use self::app::async_task_spawner;
use crate::app::AppConfig;
pub use crate::config::WebClientConfig;
use crate::window_messenger::WindowMessenger;
use percy_dom::single_page_app::{intercept_relative_links, set_onpopstate_handler};
use percy_dom::{render::create_render_scheduler, PercyDom, VirtualNode};
use routes::create_router;
use wasm_bindgen::prelude::*;

mod app;
mod config;
mod render;
mod routes;
mod view;

mod window_messenger;

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
    pub fn new_append_to_mount(config: WebClientConfig, dom_selector_of_mount: &str) -> Self {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        let render: Arc<Mutex<Box<dyn FnMut() -> ()>>> = Arc::new(Mutex::new(Box::new(|| {})));
        let render_clone = Arc::clone(&render);

        let app = create_app(config, render);

        let window_messenger = WindowMessenger::new(app.world.clone());
        setup_single_page_application(window_messenger);

        let pdom = create_percy_dom(dom_selector_of_mount, render_app(&app.world));

        let render = move || render_app(&app.world);
        let render = create_render_scheduler(pdom, render);

        *render_clone.lock().unwrap() = render;

        Self {
            rerender: render_clone,
        }
    }
}

fn create_app(
    config: WebClientConfig,
    render: Arc<Mutex<Box<dyn FnMut() -> ()>>>,
) -> PercyPreviewApp {
    PercyPreviewApp::new(
        AppConfig {
            async_task_spawner: config.async_task_spawner,
            after_path_change: Box::new(|new_path| {
                history()
                    .push_state_with_url(&JsValue::null(), "Percy Preview App", Some(new_path))
                    .unwrap();
            }),
            previews: config.previews,
        },
        render,
    )
}

fn setup_single_page_application(window_messenger: WindowMessenger) {
    window_messenger.msg_set_path(window().location().pathname().unwrap().to_string());

    let wm1 = window_messenger.clone();
    let wm2 = window_messenger;

    intercept_relative_links(move |new_path| {
        wm1.msg_set_path(new_path);
    });
    set_onpopstate_handler(move |new_path| {
        wm2.msg_set_path(new_path);
    });
}

fn create_percy_dom(dom_selector_of_mount: &str, start_view: VirtualNode) -> PercyDom {
    let mount = document()
        .query_selector(dom_selector_of_mount)
        .unwrap()
        .unwrap();
    let pdom = PercyDom::new_append_to_mount(start_view, &mount);
    pdom
}

fn window() -> web_sys::Window {
    web_sys::window().unwrap()
}

fn document() -> web_sys::Document {
    window().document().unwrap()
}

fn history() -> web_sys::History {
    window().history().unwrap()
}
