extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate virtual_dom_rs;

extern crate web_sys;
use web_sys::Element;

extern crate isomorphic_app;
use isomorphic_app::App;
use isomorphic_app::VirtualNode;

#[wasm_bindgen]
pub struct Client {
    app: App,
    root_node: Option<Element>,
    previous_vdom: Option<VirtualNode>,
}

// Expose globals from JS for things such as request animation frame
// that web sys doesn't seem to have yet
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
        let app = App::from_state_json(initial_state);

        // TODO: Use request animation frame from web_sys
        // https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Window.html#method.request_animation_frame
        app.store.borrow_mut().subscribe(Box::new(|| {
            web_sys::console::log_1(&JsValue::from("Updating state"));
            global_js.update();
        }));

        Client {
            app,
            root_node: None,
            previous_vdom: None,
        }
    }

    pub fn set_root_node(&mut self, root_node: Element) {
        self.root_node = Some(root_node);
    }

    pub fn render(&mut self) -> Element {
        let html = self.app.render();

        self.previous_vdom = Some(html);

        self.previous_vdom.as_ref().unwrap().create_element()
    }

    pub fn update_dom(&mut self) {
        let mut new_vdom = self.app.render();

        if let Some(ref previous_vdom) = self.previous_vdom {
            let patches = virtual_dom_rs::diff(&previous_vdom, &mut new_vdom);
            let root_node = self.root_node.take().unwrap();
            virtual_dom_rs::patch(root_node, &patches);
        }

        self.previous_vdom = Some(new_vdom);
    }
}
