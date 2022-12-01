use crate::event::virtual_events::ElementEventsId;
use crate::event::{EventHandler, EventName, MouseEvent, VirtualEvents, ELEMENT_EVENTS_ID_PROP};
use js_sys::Reflect;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

/// Insert a non-delegated event
pub fn insert_non_delegated_event(
    element: &web_sys::Element,
    onevent: &EventName,
    callback: &EventHandler,
    events_id: ElementEventsId,
    events: &VirtualEvents,
) {
    let events_clone = events.clone();

    let event_name = onevent.without_on_prefix();

    let on_event = onevent.with_on_prefix().to_string();
    let on_event_clone = on_event.clone();

    let callback_wrapper = move |event: web_sys::Event| {
        let this_elem = event.current_target().unwrap();

        let events_id = Reflect::get(&this_elem, &ELEMENT_EVENTS_ID_PROP.into()).unwrap();
        let events_id = events_id.as_string();
        let events_id = events_id.unwrap();

        let events_id =
            events_id.trim_start_matches(&events_clone.events_id_props_prefix().to_string());
        let events_id: u32 = events_id.parse().unwrap();

        let event_name = EventName::new(on_event_clone.clone().into());
        let cb = events_clone
            .get_event_handler(&ElementEventsId::new(events_id), &event_name)
            .unwrap();

        match cb {
            EventHandler::NoArgs(no_args) => (no_args.borrow_mut())(),
            EventHandler::MouseEvent(mouse) => {
                (mouse.borrow_mut())(MouseEvent::new(event.dyn_into().unwrap()));
            }
            EventHandler::UnsupportedSignature(cb) => {
                let cb: &js_sys::Function = cb.as_ref().as_ref().unchecked_ref();

                let context = JsValue::NULL;
                cb.call1(&context, &event).unwrap();
            }
        };
    };

    let callback_wrapper = Box::new(callback_wrapper) as Box<dyn FnMut(_) -> ()>;
    let callback_wrapper = Closure::wrap(callback_wrapper);

    let current_elem: &web_sys::EventTarget = element.dyn_ref().unwrap();
    current_elem
        .add_event_listener_with_callback(
            event_name,
            callback_wrapper.as_ref().as_ref().unchecked_ref(),
        )
        .unwrap();

    events.insert_event(
        events_id,
        onevent.clone(),
        callback.clone(),
        Some(Rc::new(callback_wrapper)),
    );
}
