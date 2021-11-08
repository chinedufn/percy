use crate::event::event_name::EventName;
use crate::event::EventHandler;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::JsValue;

/// Private type used to attach identifiers to DOM elements so that we can look up their event
/// callbacks.
#[doc(hidden)]
pub const EVENTS_ID_PROP: &'static str = "__events_id__";

type Events = Rc<RefCell<HashMap<u32, HashMap<EventName, ManagedEvent>>>>;

// Really only needs to be boxed.. but using an Rc let's us implement the
//  removes_old_non_delegated_event_listeners test.
// A future optimization could be using a feature flag to determine whether to Rc or Box this.
// i.e. #[cfg(feature = "__test-utils")]
pub(crate) type EventWrapper = Rc<dyn AsRef<JsValue>>;

/// Node's in a VirtualNode tree are indexed depth first, where the first node is index 0, it's
/// first child node is index 1, and the first child's first child is index 2.
///
/// When we create a DOM node, we store all of it's closures and all of it's children's closures
/// in this map.
///
/// We also set a `.__nodeIdx` property on nodes that have one or more events.
///
/// Percy will sometimes use event delegation, and other times attach events directly to DOM
/// elements, depending on the kind of event.
///
/// The `.__nodeIdx` property is used to power event delegation, so that the main event handler can
/// look up the callback.
///
/// ## Cloning
///
/// EventsByNodeIdx can be cheaply cloned and passed around.
/// Clones share the same inner data.
#[derive(Clone)]
pub struct EventsByNodeIdx {
    events: Events,
    // Never changes after creation.
    events_id_props_prefix: f64,
}

/// An event that to be managed by the PercyDom.
pub enum ManagedEvent {
    /// Every kind of delegated event, such as onclick, has a single event listener attached to
    /// the PercyDom's mount.
    /// When that listener is called, it looks up the proper ManagedEvent::Delegated event to call.
    Delegated(EventHandler),
    /// For non delegated events, an event listener is attached to the DOM element using
    /// .add_event_listener();
    /// That event listener is an `EventWrapper`, which in turn will find and call the
    /// `EventAttribFn`.
    /// This setup allows us to replace the `EventAttribFn` after every render without needing
    /// to re-attach event listeners.
    NonDelegated(EventHandler, EventWrapper),
}

impl ManagedEvent {
    fn is_delegated(&self) -> bool {
        matches!(self, ManagedEvent::Delegated(_))
    }
}

impl EventsByNodeIdx {
    /// Create a new EventsByNodeIdx.
    pub fn new() -> Self {
        EventsByNodeIdx {
            events: Rc::new(RefCell::new(Default::default())),
            events_id_props_prefix: js_sys::Math::random(),
        }
    }

    /// Unique for every PercyDom so that if multiple instances of PercyDom are nested their
    /// event delegation handlers don't collide.
    pub fn events_id_props_prefix(&self) -> f64 {
        self.events_id_props_prefix
    }

    /// Insert a newly tracked event.
    ///
    /// # Panics
    ///
    /// Panics if the event_name is delegated and the event is not, or vice versa.
    pub fn insert_managed_event(&self, node_idx: u32, event_name: EventName, event: ManagedEvent) {
        assert_eq!(event_name.is_delegated(), event.is_delegated());

        self.events
            .borrow_mut()
            .entry(node_idx)
            .or_default()
            .insert(event_name, event);
    }

    /// Insert a newly tracked event.
    ///
    /// # Panics
    ///
    /// Panics if there isn't an event attrib fn to overwrite.
    pub fn overwrite_event_attrib_fn(
        &self,
        node_idx: u32,
        event_name: &EventName,
        event: EventHandler,
    ) {
        let mut events = self.events.borrow_mut();

        let func = match events
            .entry(node_idx)
            .or_default()
            .get_mut(event_name)
            .unwrap()
        {
            ManagedEvent::Delegated(func) => func,
            ManagedEvent::NonDelegated(func, _) => func,
        };

        *func = event;
    }

    /// Remove all of the events from one node ID and add them to another node ID.
    pub fn move_events(&mut self, old_node_id: &u32, new_node_id: u32) {
        let mut events = self.events.borrow_mut();

        if let Some(old_events) = events.remove(old_node_id) {
            events.insert(new_node_id, old_events);
        }
    }

    /// Remove a managed event.
    pub fn remove_managed_event(&mut self, node_id: &u32, event_name: &EventName) -> ManagedEvent {
        self.events
            .borrow_mut()
            .get_mut(node_id)
            .unwrap()
            .remove(event_name)
            .unwrap()
    }

    /// Get the event handler for a node.
    pub fn get_event_handler(&self, node_id: &u32, event_name: &EventName) -> Option<EventHandler> {
        self.events.borrow().get(node_id)?.get(event_name).map(|e| {
            let event = match e {
                ManagedEvent::Delegated(e) => e,
                ManagedEvent::NonDelegated(e, _) => e,
            };

            event.clone()
        })
    }

    // Used by a percy-dom test.
    // TODO: Put this behind #[cfg(feature = "__test-utils")]
    #[doc(hidden)]
    pub fn __get_event_wrapper_clone(&self, node_id: &u32, event_name: &EventName) -> EventWrapper {
        self.events
            .borrow()
            .get(node_id)
            .unwrap()
            .get(event_name)
            .map(|e| match e {
                ManagedEvent::NonDelegated(_, wrapper) => wrapper.clone(),
                _ => panic!(),
            })
            .unwrap()
    }

    /// Remove an event handler.
    pub fn remove_event_handler(
        &self,
        node_id: &u32,
        event_name: &EventName,
    ) -> Option<ManagedEvent> {
        self.events
            .borrow_mut()
            .get_mut(node_id)?
            .remove(event_name)
    }

    /// Remove all event handlers for a node.
    pub fn remove_node(&self, node_id: &u32) {
        self.events.borrow_mut().remove(node_id);
    }
}
