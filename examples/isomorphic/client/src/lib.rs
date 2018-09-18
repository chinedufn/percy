extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate virtual_dom_rs;

extern crate web_sys;
use web_sys::Element;

extern crate isomorphic_app;
use isomorphic_app::App;
use isomorphic_app::VirtualNode;

#[wasm_bindgen(module = "./src/client.js")]
extern "C" {
    pub fn update();
}

#[wasm_bindgen]
pub struct Client {
    app: App,
    root_node: Option<Element>,
    previous_vdom: Option<VirtualNode>,
}

#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new(initial_state: &str) -> Client {
        let app = App::from_state_json(initial_state);

        // TODO: Try using a wasm-bindgen closure and an extern request_animation_frame
        // instead of using this `update()` method for request animation frame
        app.state.borrow_mut().subscribe(Box::new(|| {
            update();
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
