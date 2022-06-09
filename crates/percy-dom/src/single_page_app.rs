//! Utilities for single page applications.

use js_sys::Reflect;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::Url;

/// Ensures that anytime a link such as `<a href="/foo" />` is clicked we call the provided
/// callback with "/foo".
pub fn intercept_relative_links<F: FnMut(String) -> () + 'static>(mut on_anchor_tag_click: F) {
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

                on_anchor_tag_click(link_url.pathname())
            }
        }
    };
    let on_anchor_click = Closure::wrap(Box::new(on_anchor_click) as Box<dyn FnMut(_)>);

    window()
        .add_event_listener_with_callback("click", on_anchor_click.as_ref().unchecked_ref())
        .unwrap();
    on_anchor_click.forget();
}

/// Set a function to be called with the new path whenever a popstate event occurs on the window.
pub fn set_onpopstate_handler<F: FnMut(String) -> () + 'static>(mut on_new_path: F) {
    let on_popstate = move |_: web_sys::Event| {
        let location = location();
        let path = location.pathname().unwrap() + &location.search().unwrap();

        on_new_path(path)
    };
    let on_popstate = Box::new(on_popstate) as Box<dyn FnMut(_)>;
    let on_popstate = Closure::wrap(on_popstate);
    window().set_onpopstate(Some(on_popstate.as_ref().unchecked_ref()));
    on_popstate.forget();
}

fn window() -> web_sys::Window {
    web_sys::window().unwrap()
}

fn document() -> web_sys::Document {
    window().document().unwrap()
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
