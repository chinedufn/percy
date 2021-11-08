//! Various tests that ensure that we properly handle events.
//!
//! To run all tests in this file:
//!
//! wasm-pack test --chrome --headless crates/percy-dom --test events

use crate::testing_utilities::{create_mount, document, get_element_by_id, random_id};
use percy_dom::event::{EventHandler, EventName, EventsByNodeIdx, ManagedEvent, EVENTS_ID_PROP};
use percy_dom::prelude::*;
use percy_dom::{Patch, PercyDom, VElement};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

mod testing_utilities;

/// Verify that the oninput event works.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- input_event_works
#[wasm_bindgen_test]
fn input_event_works() {
    let id = random_id();
    let text = start_text();

    let mount = create_mount();
    let _pdom = PercyDom::new_replace_mount(
        input_node_with_events(id, vec![EventName::ONINPUT], text.clone(), APPEND_TEXT_ONE),
        mount,
    );

    assert_text_unmodified(&text);
    send_input_event(id);
    assert_text_appended(&text, APPEND_TEXT_ONE);
}

/// Verify that the onclick event works.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- click_event_works
#[wasm_bindgen_test]
fn click_event_works() {
    let id = random_id();
    let text = start_text();

    let mount = create_mount();
    let _pdom = PercyDom::new_replace_mount(
        div_node_with_event(id, vec![EventName::ONCLICK], text.clone(), APPEND_TEXT_ONE),
        mount,
    );

    assert_text_unmodified(&text);
    send_click_event(id);
    assert_text_appended(&text, APPEND_TEXT_ONE);
}

/// Verify that if we patch over an element with a delegated event we call the new event.
///
/// This ensures that we're always using the latest closures for an element, which is important for
/// closures that capture values since these values can be different across different renders.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- updated_non_delegated_event_handler
#[wasm_bindgen_test]
fn updated_non_delegated_event_handler() {
    let event = EventName::ONINPUT;
    assert_eq!(event.is_delegated(), false);

    let id = random_id();
    let text = start_text();

    let mount = create_mount();
    let mut pdom = PercyDom::new_replace_mount(
        input_node_with_events(id, vec![event.clone()], text.clone(), APPEND_TEXT_ONE),
        mount,
    );
    pdom.update(input_node_with_events(
        id,
        vec![event.clone()],
        text.clone(),
        APPEND_TEXT_TWO,
    ));

    assert_text_unmodified(&text);
    send_input_event(id);
    assert_text_appended(&text, APPEND_TEXT_TWO);
}

/// Verify that if we patch over an element with a non-delegated event the new callback is properly
/// called.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- updated_delegated_event_handler
#[wasm_bindgen_test]
fn updated_delegated_event_handler() {
    let event = EventName::ONCLICK;
    assert!(event.is_delegated());

    let id = random_id();
    let text = start_text();

    let mount = create_mount();
    let mut pdom = PercyDom::new_replace_mount(
        div_node_with_event(id, vec![event.clone()], text.clone(), APPEND_TEXT_ONE),
        mount,
    );
    pdom.update(div_node_with_event(
        id,
        vec![event.clone()],
        text.clone(),
        APPEND_TEXT_TWO,
    ));

    assert_text_unmodified(&text);
    send_click_event(id);
    assert_text_appended(&text, APPEND_TEXT_TWO);
}

/// Verify that we remove non delegated event listeners when the event is removed.
///
/// We do this by creating a non delegated event, then patching it away, then patching it back.
/// We then verify that only the final event was called, since the first one should have been
/// removed from the DOM.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- removes_old_non_delegated_event_listeners
#[wasm_bindgen_test]
fn removes_old_non_delegated_event_listeners() {
    let event = EventName::new("onfoobar".into());
    assert_eq!(event.is_delegated(), false);

    let id = random_id();
    let text = start_text();

    let one = input_node_with_events(id, vec![event.clone()], text.clone(), APPEND_TEXT_ONE);
    let two = html! { <input id=id />};
    let three = input_node_with_events(id, vec![event.clone()], text.clone(), APPEND_TEXT_ONE);

    let mount = create_mount();
    let mut pdom = PercyDom::new_replace_mount(one, mount);

    // We hold onto the old closures so that they don't get invalidated.
    let old_event = pdom.events.__get_event_wrapper_clone(&0, &event);

    pdom.update(two);
    pdom.update(three);

    assert_text_unmodified(&text);
    send_foobar_event(id);
    assert_text_appended(&text, APPEND_TEXT_ONE);

    drop(old_event);
}

/// Verify that we properly set the `__events_id__` on newly created elements and their children.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- sets_events_id_on_created_elements
#[wasm_bindgen_test]
fn sets_events_id_on_created_elements() {
    let node_idx = 10;

    let vnode: VirtualNode = html! {
        <div id="outer" oninput=||{}>
          <button id="inner" oninput=||{}></button>
          <button id="no-events"></button>
        </div>
    };
    let mut events = EventsByNodeIdx::new();
    let node = vnode.create_dom_node(node_idx, &mut events);
    document().body().unwrap().append_child(&node).unwrap();

    let outer_events_id =
        js_sys::Reflect::get(&get_element_by_id("outer"), &EVENTS_ID_PROP.into()).unwrap();
    let expected_outer_events_id: JsValue =
        format!("{}{}", events.events_id_props_prefix(), node_idx).into();

    let inner_events_id =
        js_sys::Reflect::get(&get_element_by_id("inner"), &EVENTS_ID_PROP.into()).unwrap();
    let expected_inner_events_id: JsValue =
        format!("{}{}", events.events_id_props_prefix(), node_idx + 1).into();

    let no_events =
        js_sys::Reflect::get(&get_element_by_id("no-events"), &EVENTS_ID_PROP.into()).unwrap();

    assert_eq!(outer_events_id, expected_outer_events_id);
    assert_eq!(inner_events_id, expected_inner_events_id);
    assert_eq!(no_events, JsValue::UNDEFINED);
}

/// Verify that properly handle the patch that sets the __events_id__ property.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- patch_set_events_id
#[wasm_bindgen_test]
fn patch_set_events_id() {
    let node: VirtualNode = html! {
        <div id=random_id() onclick=||{}></div>
    };
    let mut events = EventsByNodeIdx::new();

    let elem = node.create_dom_node(0, &mut events);

    let patch = Patch::SetEventsId {
        old_idx: 0,
        new_idx: 99,
    };

    percy_dom::patch(
        elem.clone(),
        &VirtualNode::text("..."),
        &mut events,
        &[patch],
    )
    .unwrap();

    let events_id = js_sys::Reflect::get(&elem, &EVENTS_ID_PROP.into()).unwrap();
    let events_id = events_id.as_string().unwrap();
    let events_id = events_id.trim_start_matches(&events.events_id_props_prefix().to_string());
    assert_eq!(events_id, "99");
}

/// Verify that if we apply a patch to set the node's events_id, we move all of its existing events
/// over to the new ID.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- set_events_id_moves_events
#[wasm_bindgen_test]
fn set_events_id_moves_events() {
    let node: VirtualNode = html! {
        <div id=random_id() onclick=||{}></div>
    };
    let mut events = EventsByNodeIdx::new();
    events.insert_managed_event(
        0,
        EventName::ONCLICK,
        ManagedEvent::Delegated(EventHandler::NoArgs(Rc::new(RefCell::new(|| {})))),
    );

    let elem = node.create_dom_node(0, &mut events);

    let patch = Patch::SetEventsId {
        old_idx: 0,
        new_idx: 99,
    };

    percy_dom::patch(
        elem.clone(),
        &VirtualNode::text("..."),
        &mut events,
        &[patch],
    )
    .unwrap();

    assert!(events.get_event_handler(&99, &EventName::ONCLICK).is_some());
}

/// Verify that when replacing a node we set the __events_id__ property using the new_idx, not
/// the old_idx.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- replaced_element_gets_new_node_idx_as_events_id
#[wasm_bindgen_test]
fn replaced_element_gets_new_node_idx_as_events_id() {
    let node: VirtualNode = html! {
        <div id=random_id() onclick=||{}></div>
    };
    let mut events = EventsByNodeIdx::new();

    let old_elem = node.create_dom_node(0, &mut events);
    document().body().unwrap().append_child(&old_elem).unwrap();

    let id = random_id();
    let patch = Patch::Replace {
        old_idx: 0,
        new_idx: 99,
        new_node: &html! { <div id=id onclick=|| {}></div> },
    };

    percy_dom::patch(
        old_elem.clone(),
        &VirtualNode::text("..."),
        &mut events,
        &[patch],
    )
    .unwrap();

    let new_elem = get_element_by_id(id);
    let events_id = js_sys::Reflect::get(&new_elem, &EVENTS_ID_PROP.into()).unwrap();
    assert_eq!(
        events_id.as_string().unwrap(),
        format!("{}{}", events.events_id_props_prefix(), 99)
    );
}

/// Verify that when replacing a text node we set the __events_id__ property using the new_idx, not
/// the old_idx.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- replaced_text_gets_new_node_idx_as_events_id
#[wasm_bindgen_test]
fn replaced_text_gets_new_node_idx_as_events_id() {
    let node: VirtualNode = html! {
        <div>
            Some text
        </div>
    };
    let mut events = EventsByNodeIdx::new();

    let old_elem = node.create_dom_node(0, &mut events);
    document().body().unwrap().append_child(&old_elem).unwrap();

    let id = random_id();
    let patch = Patch::Replace {
        old_idx: 1,
        new_idx: 55,
        new_node: &html! {
            <em id=id onclick=||{}></em>
        },
    };

    percy_dom::patch(
        old_elem.clone(),
        &VirtualNode::text("..."),
        &mut events,
        &[patch],
    )
    .unwrap();

    let new_elem = get_element_by_id(id);
    let events_id = js_sys::Reflect::get(&new_elem, &EVENTS_ID_PROP.into()).unwrap();
    assert_eq!(
        events_id.as_string().unwrap(),
        format!("{}{}", events.events_id_props_prefix(), 55)
    );
}

/// Verify that when appending a child node we set the __events_id__ property using the new_idx,
/// not the old_idx.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- append_child_node_gets_new_node_idx_as_events_id
#[wasm_bindgen_test]
fn append_child_node_gets_new_node_idx_as_events_id() {
    let node: VirtualNode = html! {
        <div id=random_id() onclick=||{}></div>
    };
    let mut events = EventsByNodeIdx::new();

    let elem = node.create_dom_node(0, &mut events);
    document().body().unwrap().append_child(&elem).unwrap();

    let appended_id = random_id();
    let child = html! { <div id=appended_id onclick=||{}> </div> };
    let patch = Patch::AppendChildren {
        old_idx: 0,
        new_nodes: vec![(99, &child)],
    };

    percy_dom::patch(
        elem.clone(),
        &VirtualNode::text("..."),
        &mut events,
        &[patch],
    )
    .unwrap();

    let appended_elem = get_element_by_id(&appended_id);
    let events_id = js_sys::Reflect::get(&appended_elem, &EVENTS_ID_PROP.into()).unwrap();
    assert_eq!(
        events_id.as_string().unwrap(),
        format!("{}{}", events.events_id_props_prefix(), 99)
    );
}

/// Verify that properly handle the patch that removes the __events_id__ property.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- patch_remove_events_id
#[wasm_bindgen_test]
fn patch_remove_events_id() {
    let node: VirtualNode = html! {
        <div id=random_id() onclick=||{}></div>
    };
    let mut events = EventsByNodeIdx::new();

    let elem = node.create_dom_node(0, &mut events);
    js_sys::Reflect::set(&elem, &EVENTS_ID_PROP.into(), &"...".into()).unwrap();

    let patch = Patch::RemoveEventsId(0);

    percy_dom::patch(
        elem.clone(),
        &VirtualNode::text("..."),
        &mut events,
        &[patch],
    )
    .unwrap();

    let events_id = js_sys::Reflect::get(&elem, &EVENTS_ID_PROP.into()).unwrap();
    assert!(events_id.as_string().is_none());
}

/// Verify that if we patch over an element that has no events with an element that has a
/// non-delegated event the event works.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- patch_add_non_delegated_event_listener
#[wasm_bindgen_test]
fn patch_add_non_delegated_event_listener() {
    let event = EventName::ONINPUT;
    assert_eq!(event.is_delegated(), false);

    let id = random_id();
    let text = start_text();

    let mount = create_mount();
    let mut pdom = PercyDom::new_replace_mount(VirtualNode::element("input"), mount);
    pdom.update(input_node_with_events(
        id,
        vec![event.clone()],
        text.clone(),
        APPEND_TEXT_TWO,
    ));

    assert_text_unmodified(&text);
    send_input_event(id);
    assert_text_appended(&text, APPEND_TEXT_TWO);
}

/// Verify that if we patch over an element that has a non-delegated event with an element that has no
/// events, we remove that event from the events store.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- patch_remove_non_delegated_event_listener
#[wasm_bindgen_test]
fn patch_remove_non_delegated_event_listener() {
    let event = EventName::ONINPUT;
    assert_eq!(event.is_delegated(), false);

    let id = random_id();
    let text = start_text();

    let mount = create_mount();
    let mut pdom = PercyDom::new_replace_mount(
        input_node_with_events(id, vec![event.clone()], text.clone(), APPEND_TEXT_TWO),
        mount,
    );

    assert!(pdom.events.get_event_handler(&0, &event).is_some());
    pdom.update(html! { <input id=id onclick=|| {} />});
    assert!(pdom.events.get_event_handler(&0, &event).is_none());

    assert_text_unmodified(&text);
    send_input_event(id);
    assert_text_unmodified(&text);
}

/// Verify that if we patch over an element that has no events with an element that has a
/// delegated event the event works.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- patch_add_delegated_event_listener
#[wasm_bindgen_test]
fn patch_add_delegated_event_listener() {
    let event = EventName::ONCLICK;
    assert!(event.is_delegated());

    let id = random_id();
    let text = start_text();

    let mount = create_mount();
    let mut pdom = PercyDom::new_replace_mount(VirtualNode::element("div"), mount);
    pdom.update(div_node_with_event(
        id,
        vec![event.clone()],
        text.clone(),
        APPEND_TEXT_TWO,
    ));

    assert_text_unmodified(&text);
    send_click_event(id);
    assert_text_appended(&text, APPEND_TEXT_TWO);
}

/// Verify that if we patch over an element that has a delegated event with an element that has no
/// events, we remove that event from the events store.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- patch_remove_delegated_event_listener
#[wasm_bindgen_test]
fn patch_remove_delegated_event_listener() {
    let event = EventName::ONCLICK;
    assert!(event.is_delegated());

    let id = random_id();
    let text = start_text();

    let mount = create_mount();
    let mut pdom = PercyDom::new_replace_mount(
        div_node_with_event(id, vec![event.clone()], text.clone(), APPEND_TEXT_TWO),
        mount,
    );

    assert!(pdom.events.get_event_handler(&0, &event).is_some());
    pdom.update(html! { <div id=id oninput=||{}></div>});
    assert!(pdom.events.get_event_handler(&0, &event).is_none());

    assert_text_unmodified(&text);
    send_click_event(id);
    assert_text_unmodified(&text);
}

/// Verify that our patch for removing all managed events at a given node idx works.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- patch_remove_all_events_with_node_idx
#[wasm_bindgen_test]
fn patch_remove_all_events_with_node_idx() {
    let mut events = EventsByNodeIdx::new();
    events.insert_managed_event(
        0,
        EventName::ONCLICK,
        ManagedEvent::Delegated(EventHandler::UnsupportedSignature(EventAttribFn(Rc::new(
            JsValue::NULL,
        )))),
    );
    let patch = Patch::RemoveAllManagedEventsWithNodeIdx(0);

    let node = VirtualNode::element("div").create_dom_node(0, &mut events);
    percy_dom::patch(node, &VirtualNode::text("..."), &mut events, &[patch]).unwrap();

    assert!(events
        .get_event_handler(&123, &EventName::ONCLICK)
        .is_none());
}

/// Verify that we can create a closure that does not have any arguments.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- closure_with_no_arguments
#[wasm_bindgen_test]
fn closure_with_no_arguments() {
    let _ = html! {
        <div onclick=|| {}></div>
    };
}

/// Verify that we can call an event with it's event argument.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- closure_with_no_arguments
#[wasm_bindgen_test]
fn closure_with_arguments() {
    let called = Rc::new(Cell::new(false));
    let called_clone = called.clone();

    let id = random_id();

    let node: VirtualNode = html! {
        <input
          id=id
          oninput=move |event: web_sys::InputEvent| {
               let input_elem = event.target().unwrap();
               let _input_elem = input_elem.dyn_into::<web_sys::HtmlInputElement>().unwrap();
               called_clone.set(true);
          }
        />
    };
    let mount = create_mount();
    let _pdom = PercyDom::new_replace_mount(node, mount);

    assert_eq!(called.get(), false);
    send_input_event(id);
    assert_eq!(called.get(), true);
}

/// Verify that our event delegation bubbles up to parent elements.
/// We do this by clicking on a child element and verifying that the parent element's onclick
/// handler is triggered.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- delegated_child_events_propagates_to_parent
#[wasm_bindgen_test]
fn delegated_child_events_propagates_to_parent() {
    assert!(EventName::ONCLICK.is_delegated());

    let called = Rc::new(Cell::new(false));
    let called_clone = called.clone();

    let id = random_id();

    let node: VirtualNode = html! {
        <div
          onclick=move |_event: percy_dom::event::MouseEvent| {
               called_clone.set(true);
          }
        >
          <div>
            <span id=id></span>
          </div>
        </div>
    };
    let mount = create_mount();
    let _pdom = PercyDom::new_replace_mount(node, mount);

    assert_eq!(called.get(), false);
    send_click_event(id);
    assert_eq!(called.get(), true);
}

/// Verify that stop propagation works on delegated events.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test events -- stop_propagation_on_delegated_event
#[wasm_bindgen_test]
fn stop_propagation_on_delegated_event() {
    assert!(EventName::ONCLICK.is_delegated());

    let called = Rc::new(Cell::new(false));
    let called_clone = called.clone();

    let id = random_id();

    let node: VirtualNode = html! {
        <div
          onclick=move |_event: percy_dom::event::MouseEvent| {
               called_clone.set(true);
          }
        >
          <span id=id
             onclick = |event: percy_dom::event::MouseEvent| {
               event.stop_propagation();
             }
           ></span>
        </div>
    };
    let mount = create_mount();
    let _pdom = PercyDom::new_replace_mount(node, mount);

    assert_eq!(called.get(), false);
    send_click_event(id);
    assert_eq!(called.get(), false);
}

fn input_node_with_events(
    id: &str,
    events: Vec<EventName>,
    text: Rc<RefCell<String>>,
    append: &'static str,
) -> VirtualNode {
    node_with_events("input", id, events, text, append)
}

fn div_node_with_event(
    id: &str,
    events: Vec<EventName>,
    text: Rc<RefCell<String>>,
    append: &'static str,
) -> VirtualNode {
    node_with_events("div", id, events, text, append)
}

fn node_with_events(
    tag: &str,
    id: &str,
    events: Vec<EventName>,
    text: Rc<RefCell<String>>,
    append: &'static str,
) -> VirtualNode {
    let mut elem = VElement::new(tag);
    elem.attrs.insert("id".to_string(), id.into());

    for event in events {
        let text = text.clone();
        let closure = move || {
            append_text(&text, append);
        };

        elem.events
            .insert(event, EventHandler::NoArgs(Rc::new(RefCell::new(closure))));
    }

    VirtualNode::Element(elem)
}

const START_TEXT: &'static str = "Start Text";
const APPEND_TEXT_ONE: &'static str = "- append1";
const APPEND_TEXT_TWO: &'static str = "- append2";

fn start_text() -> Rc<RefCell<String>> {
    Rc::new(RefCell::new(START_TEXT.to_string()))
}

fn append_text(text: &Rc<RefCell<String>>, append: &str) {
    *text.borrow_mut() = format!("{}{}", text.borrow().as_str(), append);
}

fn assert_text_unmodified(text: &Rc<RefCell<String>>) {
    assert_eq!(
        text.borrow().as_str(),
        START_TEXT,
        "Text should not have changed."
    );
}

fn assert_text_appended(text: &Rc<RefCell<String>>, append: &str) {
    assert_eq!(
        text.borrow().as_str(),
        format!("{}{}", START_TEXT, append),
        "Text should have been appended changed"
    );
}

fn send_input_event(id: &str) {
    send_event::<web_sys::HtmlInputElement>(id, &web_sys::InputEvent::new("input").unwrap());
}

fn send_foobar_event(id: &str) {
    let event = web_sys::Event::new("foobar").unwrap();
    send_event::<web_sys::Element>(id, &event);
}

fn send_click_event(id: &str) {
    let mouse_event = web_sys::MouseEvent::new("click").unwrap();
    mouse_event.init_mouse_event_with_can_bubble_arg("click", true);

    send_event::<web_sys::HtmlElement>(id, &virtual_node::event::MouseEvent::new(mouse_event));
}

fn send_event<T>(elem_id: &str, event: &web_sys::Event)
where
    T: JsCast,
    web_sys::EventTarget: From<T>,
{
    let elem: T = get_element_by_id(elem_id).dyn_into().unwrap();

    web_sys::EventTarget::from(elem)
        .dispatch_event(&event)
        .unwrap();
}
