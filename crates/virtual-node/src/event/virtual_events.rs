use crate::event::event_name::EventName;
use crate::event::EventHandler;
use js_sys::Reflect;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::JsValue;

// Every real DOM element that we create gets a property set on it that can be used to look up
// its events in [`crate::VirtualEvents`].
#[doc(hidden)]
pub const ELEMENT_EVENTS_ID_PROP: &'static str = "__events_id__";

/// Uniquely identifies an element so that we can store it's events in [`VirtualEvents`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ElementEventsId(u32);

impl ElementEventsId {
    /// Create a new ElementEventsId.
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the inner u32 id.
    pub fn get(&self) -> u32 {
        self.0
    }
}

// Really only needs to be boxed.. but using an Rc let's us implement the
//  removes_old_non_delegated_event_listeners test.
// A future optimization could be using a feature flag to determine whether to Rc or Box this.
// i.e. #[cfg(feature = "__test-utils")]
pub(crate) type EventWrapper = Rc<dyn AsRef<JsValue>>;

/// When we create a DOM node, we store all of it's closures and all of it's children's closures
/// in VirtualEvents.
///
/// When an element gets interacted with in the DOM it's event handlers get looked up in
/// VirtualEvents.
///
/// This helps power event delegation, where for many events kinds of events such as onclick we use
/// a single event listener on the element that the application was mounted on and then as events
/// occur we look up the event handlers in VirtualEvents.
///
/// This is faster since instead of needing to add and remove event listeners from the DOM after
/// when applying patches we can simply overwrite the old closures in VirtualEvents with new ones.
///
/// ## Cloning
///
/// VirtualEvents can be cloned cheaply. Clones share the same inner data.
#[derive(Clone)]
pub struct VirtualEvents {
    inner: Rc<RefCell<VirtualEventsInner>>,
    // Never changes after creation.
    events_id_props_prefix: f64,
}
struct VirtualEventsInner {
    root: Rc<RefCell<VirtualEventNode>>,
    events: HashMap<ElementEventsId, Rc<RefCell<HashMap<EventName, EventHandler>>>>,
    /// For non delegated events an event listener is attached to the DOM element using
    /// .add_event_listener();
    /// That event listener is an `EventWrapper`, which in turn will find and call the
    /// `EventHandler`.
    /// This setup allows us to replace the `EventHandler` after every render without needing
    /// to re-attach event listeners.
    non_delegated_event_wrappers: HashMap<ElementEventsId, HashMap<EventName, EventWrapper>>,
    next_events_id: u32,
}

/// A tree where each entry holds the events for the corresponding entry in a
/// [`crate::VirtualNode`] tree.
#[derive(Debug)]
pub enum VirtualEventNode {
    Element(VirtualEventElement),
    /// Text nodes cannot have events.
    Text,
}
/// A virtual event element node.
#[derive(Debug)]
pub struct VirtualEventElement {
    events_id: ElementEventsId,
    children: Vec<Rc<RefCell<VirtualEventNode>>>,
}

impl VirtualEvents {
    /// Create a new EventsByNodeIdx.
    pub fn new() -> Self {
        VirtualEvents {
            inner: Rc::new(RefCell::new(VirtualEventsInner::new())),
            events_id_props_prefix: js_sys::Math::random(),
        }
    }

    /// Unique for every PercyDom so that if multiple instances of PercyDom are nested their
    /// event delegation handlers don't collide.
    pub fn events_id_props_prefix(&self) -> f64 {
        self.events_id_props_prefix
    }

    /// Get the root event node.
    pub fn root(&self) -> Rc<RefCell<VirtualEventNode>> {
        self.borrow().root.clone()
    }

    /// Set the root event node.
    pub fn set_root(&self, root: VirtualEventNode) {
        *self.borrow_mut().root.borrow_mut() = root;
    }

    /// Insert a newly tracked event.
    ///
    /// # Panics
    ///
    /// Panics if the event_name is delegated and the event is not, or vice versa.
    pub fn insert_event(
        &self,
        events_id: ElementEventsId,
        event_name: EventName,
        event: EventHandler,
        wrapper: Option<EventWrapper>,
    ) {
        assert_eq!(event_name.is_delegated(), wrapper.is_none());

        let mut borrow = self.borrow_mut();

        borrow
            .events
            .entry(events_id)
            .or_default()
            .borrow_mut()
            .insert(event_name.clone(), event);

        if let Some(wrapper) = wrapper {
            borrow
                .non_delegated_event_wrappers
                .entry(events_id)
                .or_default()
                .insert(event_name, wrapper);
        }
    }

    /// Overwrite an event handler.
    ///
    /// # Panics
    ///
    /// Panics if there isn't an event attrib fn to overwrite.
    pub fn overwrite_event_attrib_fn(
        &self,
        events_id: &ElementEventsId,
        event_name: &EventName,
        event: EventHandler,
    ) {
        let mut borrow = self.borrow_mut();

        let borrow = borrow.events.get_mut(events_id).unwrap();
        let mut borrow = borrow.borrow_mut();
        let func = borrow.get_mut(event_name).unwrap();

        *func = event;
    }

    /// Remove a managed event.
    pub fn remove_non_delegated_event_wrapper(
        &mut self,
        events_id: &ElementEventsId,
        event_name: &EventName,
    ) -> EventWrapper {
        let mut borrow = self.borrow_mut();
        borrow
            .non_delegated_event_wrappers
            .get_mut(events_id)
            .unwrap()
            .remove(event_name)
            .unwrap()
    }

    /// Get the event handler for a node.
    pub fn get_event_handler(
        &self,
        events_id: &ElementEventsId,
        event_name: &EventName,
    ) -> Option<EventHandler> {
        let borrow = self.borrow();
        let borrow = borrow.events.get(events_id)?;
        let borrow = borrow.borrow();
        borrow.get(event_name).cloned()
    }

    /// Remove an event handler.
    pub fn remove_event_handler(
        &self,
        events_id: &ElementEventsId,
        event_name: &EventName,
    ) -> Option<EventHandler> {
        let mut borrow = self.borrow_mut();

        let borrow = borrow.events.get_mut(events_id)?;
        let mut borrow = borrow.borrow_mut();
        borrow.remove(event_name)
    }

    /// Remove all event handlers for a node.
    pub fn remove_node(&self, events_id: &ElementEventsId) {
        let mut borrow = self.borrow_mut();
        borrow.events.remove(events_id);
        borrow.non_delegated_event_wrappers.remove(events_id);
    }

    /// Create an ElementEventsId that is unique to this VirtualEvents instance.
    pub(crate) fn unique_events_id(&self) -> ElementEventsId {
        let mut borrow = self.borrow_mut();
        let counter = borrow.next_events_id;

        borrow.next_events_id += 1;

        ElementEventsId(counter)
    }

    fn borrow(&self) -> Ref<'_, VirtualEventsInner> {
        self.inner.borrow()
    }
    fn borrow_mut(&self) -> RefMut<'_, VirtualEventsInner> {
        self.inner.borrow_mut()
    }
}

impl VirtualEventsInner {
    fn new() -> Self {
        Self {
            // ::Text will get replaced with an element shortly after creating VirtualEvents.
            root: Rc::new(RefCell::new(VirtualEventNode::Text)),
            events: HashMap::new(),
            non_delegated_event_wrappers: HashMap::new(),
            next_events_id: 0,
        }
    }
}

impl VirtualEventNode {
    /// Create a new [`VirtualEventNode::Element`].
    pub fn new_element(events_id: ElementEventsId) -> Self {
        Self::Element(VirtualEventElement::new(events_id))
    }

    /// Get the [`VirtualEventNode::VirtualEventElement`] variant.
    pub fn as_element(&self) -> Option<&VirtualEventElement> {
        match self {
            VirtualEventNode::Element(e) => Some(e),
            _ => None,
        }
    }

    /// Get a mutable reference to the [`VirtualEventNode::VirtualEventElement`] variant.
    pub fn as_element_mut(&mut self) -> Option<&mut VirtualEventElement> {
        match self {
            VirtualEventNode::Element(e) => Some(e),
            _ => None,
        }
    }
}
impl VirtualEventElement {
    /// Create a new VirtualEventNode for the given events id.
    pub fn new(events_id: ElementEventsId) -> Self {
        VirtualEventElement {
            events_id,
            children: vec![],
        }
    }

    /// Get the element's events id.
    pub fn events_id(&self) -> ElementEventsId {
        self.events_id
    }

    /// Get the element's children.
    pub fn children(&self) -> &Vec<Rc<RefCell<VirtualEventNode>>> {
        &self.children
    }

    /// Get the element's children.
    pub fn push_child(&mut self, child: VirtualEventNode) {
        self.children.push(Rc::new(RefCell::new(child)));
    }

    /// Truncate the element's children.
    pub fn truncate_children(&mut self, num_remaining: usize) {
        self.children.truncate(num_remaining);
    }
}

pub(crate) fn set_events_id(node: &JsValue, events: &VirtualEvents, events_id: ElementEventsId) {
    Reflect::set(
        &node.into(),
        &ELEMENT_EVENTS_ID_PROP.into(),
        &format!("{}{}", events.events_id_props_prefix(), events_id.get()).into(),
    )
    .unwrap();
}
