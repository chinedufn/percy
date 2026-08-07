use crate::event::virtual_events::ElementEventsId;
use crate::event::{
    EventHandler, EventName, MouseEventWebSys, VirtualEvents, ELEMENT_EVENTS_ID_PROP,
};
use js_sys::Reflect;
use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

/// A type-erased [`js_sys::Closure`]. Stored as a trait object to enable any number of arguments.
pub(crate) type EventWrapper = std::rc::Rc<dyn AsRef<wasm_bindgen::JsValue>>;

/// A handler for an event, such the closure in `onmyevent = || {}`.
#[derive(Clone)]
pub struct EventAttribFn(pub EventWrapper);

impl EventAttribFn {
    /// Create a new [`EventAttribFn`].
    pub fn new(callback: EventWrapper) -> Self {
        Self(callback)
    }
}

/// Attaches an event handler directly onto a DOM element.
///
/// See [`VirtualEvents`] and [`EventName::is_delegated`] for documentation regarding event delegation.
pub fn insert_non_delegated_event(
    element: &web_sys::Element,
    onevent: &EventName,
    callback: &EventHandler<web_sys::Window>,
    events_id: ElementEventsId,
    events: &VirtualEvents<web_sys::Window>,
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
                (mouse.borrow_mut())(MouseEventWebSys::new(event.dyn_into().unwrap()));
            }
            EventHandler::Custom(cb) => {
                use wasm_bindgen::JsCast;

                let context = wasm_bindgen::JsValue::NULL;
                let callback: &js_sys::Function = cb.as_ref().as_ref().unchecked_ref();
                callback.call1(&context, &event).unwrap();
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

    let event_wrapper = Rc::new(callback_wrapper);
    events.insert_event(
        events_id,
        onevent.clone(),
        callback.clone(),
        Some(event_wrapper),
    );
}

// Allows us to easily derive PartialEq for some of the types that contain events.
// Those PartialEq implementations are used for testing.
// Maybe we can put some of the event related PartialEq implementations
// behind a #[cfg(any(test, feature = "__test-utils"))].
impl PartialEq for EventAttribFn {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl fmt::Debug for EventAttribFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "event_handler()")
    }
}

impl From<EventWrapper> for EventAttribFn {
    fn from(inner: EventWrapper) -> Self {
        EventAttribFn(inner)
    }
}
