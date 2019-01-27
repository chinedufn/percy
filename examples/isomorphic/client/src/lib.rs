use wasm_bindgen;
use wasm_bindgen::prelude::*;

use console_error_panic_hook;
use virtual_dom_rs::prelude::*;

use web_sys;
use web_sys::Element;

use isomorphic_app;
use isomorphic_app::App;
use isomorphic_app::VirtualNode;

#[wasm_bindgen]
pub struct Client {
    app: App,
    dom_updater: DomUpdater,
}

// Expose globals from JS for things such as request animation frame
// that web sys doesn't seem to have yet
//
// TODO: Remove this and use RAF from Rust
// https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Window.html#method.request_animation_frame
#[wasm_bindgen]
extern "C" {
    pub type GlobalJS;

    pub static global_js: GlobalJS;

    #[wasm_bindgen(method)]
    pub fn update(this: &GlobalJS);
}

#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new(initial_state: &str) -> Client {
        console_error_panic_hook::set_once();

        let app = App::from_state_json(initial_state);

        // TODO: Use request animation frame from web_sys
        // https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Window.html#method.request_animation_frame
        app.store.borrow_mut().subscribe(Box::new(|| {
            web_sys::console::log_1(&JsValue::from("Updating state"));
            global_js.update();
        }));

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let root_node = document
            .get_element_by_id("isomorphic-rust-web-app")
            .unwrap();
        let dom_updater = DomUpdater::new_replace_mount(app.render(), root_node);

        Client { app, dom_updater }
    }

    pub fn render(&mut self) {
        let vdom = self.app.render();
        self.dom_updater.update(vdom);
    }
}
