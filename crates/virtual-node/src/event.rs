pub use self::event_handlers::*;
pub use self::event_name::EventName;
#[cfg(feature = "web")]
pub(crate) use self::virtual_events::set_events_id;
#[cfg(feature = "web")]
pub use self::virtual_events::VirtualEventsWebSys;
pub use self::virtual_events::{
    ElementEventsId, VirtualEventElement, VirtualEventNode, VirtualEvents, ELEMENT_EVENTS_ID_PROP,
};
#[cfg(feature = "web")]
pub use self::web::{insert_non_delegated_event, EventAttribFn};
use std::cell::RefCell;
use std::collections::hash_map::Drain;
use std::collections::HashMap;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

mod event_handlers;
mod event_name;
mod virtual_events;
#[cfg(feature = "web")]
mod web;

/// We need a custom implementation of fmt::Debug since JsValue doesn't implement debug.
pub struct Events<Dom: RealDom> {
    // TODO: Store multiple events for a given event name, not just one.
    //  `Vec<(EventName, EventHandler<Dom>>`
    events: HashMap<EventName, EventHandler<Dom>>,
}

impl<Dom: RealDom> PartialEq for Events<Dom> {
    fn eq(&self, other: &Self) -> bool {
        let Events { events: lhs_events } = self;
        let Events { events: rhs_events } = other;

        lhs_events == rhs_events
    }
}

impl<Dom: RealDom> Events<Dom> {
    /// Whether or not there is at least one event.
    pub fn has_events(&self) -> bool {
        !self.events.is_empty()
    }

    /// All of the events.
    pub fn events(&self) -> &HashMap<EventName, EventHandler<Dom>> {
        &self.events
    }

    /// Insert an event handler that does not have any arguments.
    pub fn insert_no_args(&mut self, event_name: EventName, event: Rc<RefCell<dyn FnMut()>>) {
        self.events
            .insert(event_name, EventHandler::<Dom>::NoArgs(event));
    }

    // Used by the html! macro
    #[doc(hidden)]
    pub fn __insert_unsupported_signature(
        &mut self,
        event_name: EventName,
        event: Dom::EventCallback,
    ) {
        self.events.insert(event_name, EventHandler::Custom(event));
    }

    /// Insert a mouse event handler.
    pub fn insert_mouse_event(
        &mut self,
        event_name: EventName,
        event: Rc<RefCell<dyn FnMut(Dom::MouseEvent)>>,
    ) {
        self.events
            .insert(event_name, EventHandler::MouseEvent(event));
    }

    /// Removes the element's events and returns them.
    pub fn take_events(&mut self) -> Drain<'_, EventName, EventHandler<Dom>> {
        self.events.drain()
    }

    /// Wrap the events in the given closure.
    pub fn convert_all<New: RealDom>(
        mut self,
        convert: impl Fn(EventHandler<Dom>) -> EventHandler<New>,
    ) -> Events<New> {
        let mut new_events = HashMap::with_capacity(self.events.len());

        for (event_name, before) in self.events.drain() {
            let after = convert(before);
            new_events.insert(event_name, after);
        }

        Events { events: new_events }
    }
}

impl<Dom: RealDom> Events<Dom> {
    /// Create a new Events.
    pub fn new() -> Self {
        Events {
            events: HashMap::new(),
        }
    }
}

/// In some applications, [`VirtualNode`] get converted into real DOM nodes.
///
/// This trait contains types and methods for manipulating a real DOM.
///
/// When running a client-side web application, consider using [`web_sys::Window`] as the
/// [`RealDom`].
/// When running on a server, consider using the null `()` type as the [`RealDom`].
///
/// To control how a [`VirtualNode`] gets rendered to a DOM element, implement [`RealDom`] for your
/// own custom type.
///
/// [`VirtualNode`]: crate::VirtualNode
pub trait RealDom {
    /// The event type. In the web this is [`web_sys::Event`].
    type Event;
    /// The event type for mouse events. In the web this is [`web_sys::MouseEvent`].
    type MouseEvent;
    /// The type for callbacks such as `|some_event| { ... }`.
    type EventCallback: Clone;
}

/// An [`RealDom`] implementation that uses [`web_sys`]'s event types.
#[cfg(feature = "web")]
impl RealDom for web_sys::Window {
    type Event = web_sys::Event;
    type MouseEvent = crate::event::MouseEventWebSys;
    type EventCallback = Rc<dyn AsRef<wasm_bindgen::JsValue>>;
}

impl RealDom for () {
    type Event = ();
    type MouseEvent = ();
    type EventCallback = ();
}

#[cfg(feature = "web")]
impl EventAttribFn {
    /// Currently used by `crates/percy-dom`'s test suite.
    #[doc(hidden)]
    pub fn new_noop() -> EventAttribFn {
        use wasm_bindgen::JsValue;
        let noop = Rc::new(JsValue::NULL);
        EventAttribFn::new(noop)
    }
}

impl<Dom: RealDom> fmt::Debug for Events<Dom> {
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

impl<Dom: RealDom> Deref for Events<Dom> {
    type Target = HashMap<EventName, EventHandler<Dom>>;

    fn deref(&self) -> &Self::Target {
        &self.events
    }
}

impl<Dom: RealDom> DerefMut for Events<Dom> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.events
    }
}
