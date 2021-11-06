use console_error_panic_hook;
use isomorphic_app;
use isomorphic_app::Msg;
use isomorphic_app::{App, Store};
use js_sys::Reflect;
use log::Level;
use percy_dom::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys;
use web_sys::Url;

#[wasm_bindgen]
pub struct Client {
    app: App,
    pdom: PercyDom,
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
        // In a real app you'd typically uncomment this line
        // #[cfg(debug_assertions)]
        console_log::init_with_level(Level::Debug);

        console_error_panic_hook::set_once();

        let app = App::from_state_json(initial_state);

        // TODO: Use request animation frame from web_sys
        // https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Window.html#method.request_animation_frame
        app.store.borrow_mut().subscribe(Box::new(|| {
            web_sys::console::log_1(&"Updating state".into());
            global_js.update();
        }));

        app.store.borrow_mut().set_after_route(Box::new(|new_path| {
            history().push_state_with_url(&JsValue::null(), "Rust Web App", Some(new_path));
        }));

        let store = Rc::clone(&app.store);
        let on_popstate = move |_: web_sys::Event| {
            let location = location();
            let path = location.pathname().unwrap() + &location.search().unwrap();

            store.borrow_mut().msg(&Msg::SetPath(path))
        };
        let on_popstate = Box::new(on_popstate) as Box<FnMut(_)>;
        let mut on_popstate = Closure::wrap(on_popstate);
        window().set_onpopstate(Some(on_popstate.as_ref().unchecked_ref()));
        on_popstate.forget();

        let root_node = document()
            .get_element_by_id("isomorphic-rust-web-app")
            .unwrap();
        let pdom = PercyDom::new_replace_mount(app.render(), root_node);

        let store = Rc::clone(&app.store);
        intercept_relative_links(store);

        Client { app, pdom }
    }

    pub fn render(&mut self) {
        let vdom = self.app.render();
        self.pdom.update(vdom);
    }
}

// Ensure that anytime a link such as `<a href="/foo" />` is clicked we re-render the page locally
// instead of hitting the server to load a new page.
fn intercept_relative_links(store: Rc<RefCell<Store>>) {
    let on_anchor_click = move |event: web_sys::Event| {
        // Get the tag name of the element that was clicked
        let target = event
            .target()
            .unwrap()
            .dyn_into::<web_sys::Element>()
            .unwrap();
        let tag_name = target.tag_name();
        let tag_name = tag_name.as_str();

        // If the clicked element is an anchor tag, check if it points to the current website
        // (ex: '<a href="/some-page"></a>'
        if tag_name.to_lowercase() == "a" {
            let link = Reflect::get(&target, &"href".into())
                .unwrap()
                .as_string()
                .unwrap();
            let link_url = Url::new(link.as_str()).unwrap();

            // If this was indeed a relative URL, let our single page application router
            // handle it
            if link_url.hostname() == hostname() && link_url.port() == port() {
                event.prevent_default();

                let msg = &Msg::SetPath(link_url.pathname());
                store.borrow_mut().msg(msg);
            }
        }
    };
    let on_anchor_click = Closure::wrap(Box::new(on_anchor_click) as Box<FnMut(_)>);

    window()
        .add_event_listener_with_callback("click", on_anchor_click.as_ref().unchecked_ref())
        .unwrap();
    on_anchor_click.forget();
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

fn location() -> web_sys::Location {
    document().location().unwrap()
}

fn hostname() -> String {
    location().hostname().unwrap()
}

fn port() -> String {
    location().port().unwrap()
}
