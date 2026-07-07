pub use self::event_handlers::*;
pub use self::event_name::EventName;
pub use self::non_delegated_event_wrapper::insert_non_delegated_event;
pub(crate) use self::virtual_events::set_events_id;
pub use self::virtual_events::{
    ElementEventsId, VirtualEventElement, VirtualEventNode, VirtualEvents, ELEMENT_EVENTS_ID_PROP,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::Event;

mod event_handlers;
mod event_name;
mod non_delegated_event_wrapper;
mod virtual_events;

/// A type-erased [`js_sys::Closure`]. Stored as a trait object to enable any number of arguments.
pub(crate) type EventWrapper = std::rc::Rc<dyn EventCallback>;

/// A handler for an event, such the closure in `onmyevent = || {}`.
#[derive(Clone)]
pub struct EventAttribFn(EventWrapper);

/// We need a custom implementation of fmt::Debug since JsValue doesn't implement debug.
#[derive(PartialEq)]
pub struct Events {
    events: HashMap<EventName, EventHandler>,
}

impl Events {
    /// Whether or not there is at least one event.
    pub fn has_events(&self) -> bool {
        !self.events.is_empty()
    }

    /// All of the events.
    pub fn events(&self) -> &HashMap<EventName, EventHandler> {
        &self.events
    }

    /// Insert an event handler that does not have any arguments.
    pub fn insert_no_args(&mut self, event_name: EventName, event: Rc<RefCell<dyn FnMut()>>) {
        self.events.insert(event_name, EventHandler::NoArgs(event));
    }

    // Used by the html! macro
    #[doc(hidden)]
    pub fn __insert_unsupported_signature(&mut self, event_name: EventName, event: EventWrapper) {
        self.events
            .insert(event_name, EventHandler::UnsupportedSignature(event.into()));
    }

    /// Insert a mouse event handler.
    pub fn insert_mouse_event(
        &mut self,
        event_name: EventName,
        event: Rc<RefCell<dyn FnMut(MouseEvent)>>,
    ) {
        self.events
            .insert(event_name, EventHandler::MouseEvent(event));
    }
}

impl Events {
    /// Create a new Events.
    pub fn new() -> Self {
        Events {
            events: HashMap::new(),
        }
    }
}

/// A callback that runs when an event is triggered.
///
/// For instance, this might represent the closure in `onmyevent = |_| {}`.
pub trait EventCallback {
    /// Invoke the callback.
    fn call(&self, event: &web_sys::Event);

    /// Get the underlying [`wasm_bindgen::JsValue`].
    fn as_js_value(&self) -> Option<&wasm_bindgen::JsValue>;
}

impl<T: ?Sized> EventCallback for wasm_bindgen::closure::Closure<T> {
    fn call(&self, event: &web_sys::Event) {
        use wasm_bindgen::JsCast;

        let context = wasm_bindgen::JsValue::NULL;
        let callback: &js_sys::Function = self.as_ref().as_ref().unchecked_ref();
        callback.call1(&context, &event).unwrap();
    }

    fn as_js_value(&self) -> Option<&wasm_bindgen::JsValue> {
        Some(self.as_ref())
    }
}

impl EventAttribFn {
    /// Currently used by `crates/percy-dom`'s test suite.
    #[doc(hidden)]
    pub fn new_noop() -> Self {
        struct Noop;
        impl EventCallback for Noop {
            fn call(&self, _event: &Event) {}

            fn as_js_value(&self) -> Option<&JsValue> {
                None
            }
        }
        Self(Rc::new(Noop))
    }

    pub(crate) fn call(&self, event: &web_sys::Event) {
        self.0.call(event)
    }
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

impl fmt::Debug for Events {
    // Print out all of the event names for this VirtualNode
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let events: String = self
            .events
            .keys()
            .map(|key| " ".to_string() + key.with_on_prefix())
            .collect();
        write!(f, "{}", events)
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

impl Deref for Events {
    type Target = HashMap<EventName, EventHandler>;

    fn deref(&self) -> &Self::Target {
        &self.events
    }
}

impl DerefMut for Events {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.events
    }
}
