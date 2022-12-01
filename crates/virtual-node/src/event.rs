use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub use self::event_handlers::*;
pub use self::event_name::EventName;
pub use self::non_delegated_event_wrapper::insert_non_delegated_event;
pub(crate) use self::virtual_events::set_events_id;
pub use self::virtual_events::{
    ElementEventsId, VirtualEventElement, VirtualEventNode, VirtualEvents, ELEMENT_EVENTS_ID_PROP,
};

mod event_handlers;
mod event_name;
mod non_delegated_event_wrapper;
mod virtual_events;

type EventAttribFnInner = std::rc::Rc<dyn AsRef<wasm_bindgen::JsValue>>;

/// Box<dyn AsRef<JsValue>>> is our js_sys::Closure. Stored this way to allow us to store
/// any Closure regardless of the types of its arguments.
#[derive(Clone)]
pub struct EventAttribFn(pub EventAttribFnInner);

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
    pub fn __insert_unsupported_signature(
        &mut self,
        event_name: EventName,
        event: EventAttribFnInner,
    ) {
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

impl From<EventAttribFnInner> for EventAttribFn {
    fn from(inner: EventAttribFnInner) -> Self {
        EventAttribFn(inner)
    }
}

impl Deref for EventAttribFn {
    type Target = EventAttribFnInner;

    fn deref(&self) -> &Self::Target {
        &self.0
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
