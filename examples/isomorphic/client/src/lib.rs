#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate isomorphic_app;
use isomorphic_app::App;
use isomorphic_app::Element;

#[wasm_bindgen(module = "./src/client.js")]
extern "C" {
    pub fn update();
}

#[wasm_bindgen]
pub struct Client {
    app: App,
    root_node: Option<Element>,
}

#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new(initial_state: &str) -> Client {
        let mut app = App::from_state_json(initial_state);

        // TODO: Try using a wasm-bindgen closure and an extern request_animation_frame
        // instead of using this `update()` method for request animation frame
        app.state.borrow_mut().subscribe(Box::new(|| {
            update();
        }));

        Client {
            app,
            root_node: None,
        }
    }

    pub fn set_root_node(&mut self, root_node: Element) {
        self.root_node = Some(root_node);
    }

    pub fn render(&self) -> Element {
        self.app.render().create_element()
    }

    pub fn update_dom(&mut self) {
        self.app.update_dom(&self.root_node.as_ref().unwrap())
    }
}
