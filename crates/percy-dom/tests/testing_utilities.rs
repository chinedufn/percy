//! Various helper functions and types for writing tests.

use std::cell::RefCell;
use std::rc::Rc;
use virtual_node::event::{ElementEventsId, VirtualEventNode, VirtualEvents};
use virtual_node::VirtualNode;
use wasm_bindgen::JsCast;
use web_sys::Node;

// Tests share the same DOM, so IDs need to be unique across tests.
pub fn random_id() -> &'static str {
    Box::leak(Box::new(js_sys::Math::random().to_string()))
}

pub fn document() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}

pub fn append_to_document(elem: &web_sys::Element) {
    document().body().unwrap().append_child(elem).unwrap();
}

pub fn get_element_by_id(id: &str) -> web_sys::Element {
    document().get_element_by_id(id).unwrap()
}

pub fn create_mount() -> web_sys::Element {
    let mount = document().create_element("div").unwrap();
    document().body().unwrap().append_child(&mount).unwrap();

    mount
}

pub fn create_node_and_events_and_append_to_document(vnode: VirtualNode) -> (Node, VirtualEvents) {
    let node_and_events = create_node_and_events(vnode);
    append_to_document(node_and_events.0.dyn_ref().unwrap());

    node_and_events
}

pub fn create_node_and_events(vnode: VirtualNode) -> (Node, VirtualEvents) {
    let mut events = VirtualEvents::new();
    let (node, events_node) = vnode.create_dom_node(&mut events);
    events.set_root(events_node);

    (node, events)
}

pub fn send_click_event(id: &str) {
    let mouse_event = web_sys::MouseEvent::new("click").unwrap();
    mouse_event.init_mouse_event_with_can_bubble_arg("click", true);

    send_event::<web_sys::HtmlElement>(id, &virtual_node::event::MouseEvent::new(mouse_event));
}

pub fn send_input_event(id: &str) {
    send_event::<web_sys::HtmlInputElement>(id, &web_sys::InputEvent::new("input").unwrap());
}

pub fn send_event<T>(elem_id: &str, event: &web_sys::Event)
where
    T: JsCast,
    web_sys::EventTarget: From<T>,
{
    let elem: T = get_element_by_id(elem_id).dyn_into().unwrap();

    web_sys::EventTarget::from(elem)
        .dispatch_event(&event)
        .unwrap();
}
