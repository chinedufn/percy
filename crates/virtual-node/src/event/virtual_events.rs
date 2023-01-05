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
pub struct VirtualEventNode {
    variant: VirtualEventNodeVariant,
    previous_sibling: Option<Rc<RefCell<VirtualEventNode>>>,
    next_sibling: Option<Rc<RefCell<VirtualEventNode>>>,
}

#[derive(Debug)]
enum VirtualEventNodeVariant {
    Element(VirtualEventElement),
    Text,
}

/// A virtual event element node.
#[derive(Debug)]
pub struct VirtualEventElement {
    events_id: ElementEventsId,
    children: Option<VirtualEventElementChildren>,
}
#[derive(Debug)]
struct VirtualEventElementChildren {
    first_child: Rc<RefCell<VirtualEventNode>>,
    last_child: Rc<RefCell<VirtualEventNode>>,
}

impl VirtualEvents {
    /// Create a new EventsByNodeIdx.
    pub fn new() -> Self {
        VirtualEvents {
            inner: Rc::new(RefCell::new(VirtualEventsInner::new())),
            events_id_props_prefix: js_sys::Math::random(),
        }
    }

    #[cfg(test)]
    pub fn new_with_prefix(prefix: f64) -> Self {
        VirtualEvents {
            inner: Rc::new(RefCell::new(VirtualEventsInner::new())),
            events_id_props_prefix: prefix,
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

    /// Create a new element node.
    pub fn create_element_node(&self) -> VirtualEventNode {
        VirtualEventNode {
            variant: VirtualEventNodeVariant::Element(VirtualEventElement::new(
                self.unique_events_id(),
            )),
            previous_sibling: None,
            next_sibling: None,
        }
    }

    /// Create a new element node.
    pub fn create_text_node(&self) -> VirtualEventNode {
        VirtualEventNode {
            variant: VirtualEventNodeVariant::Text,
            previous_sibling: None,
            next_sibling: None,
        }
    }

    // Create an ElementEventsId that is unique to this VirtualEvents instance.
    fn unique_events_id(&self) -> ElementEventsId {
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
        let root = VirtualEventNode {
            // ::Text will get replaced with an element shortly after creating VirtualEvents.
            variant: VirtualEventNodeVariant::Text,
            previous_sibling: None,
            next_sibling: None,
        };

        Self {
            root: Rc::new(RefCell::new(root)),
            events: HashMap::new(),
            non_delegated_event_wrappers: HashMap::new(),
            next_events_id: 0,
        }
    }
}

impl VirtualEventNode {
    /// Get the [`VirtualEventNode::VirtualEventElement`] variant.
    pub fn as_element(&self) -> Option<&VirtualEventElement> {
        match &self.variant {
            VirtualEventNodeVariant::Element(e) => Some(e),
            _ => None,
        }
    }

    /// Get a mutable reference to the [`VirtualEventNode::VirtualEventElement`] variant.
    pub fn as_element_mut(&mut self) -> Option<&mut VirtualEventElement> {
        match &mut self.variant {
            VirtualEventNodeVariant::Element(e) => Some(e),
            _ => None,
        }
    }

    /// Get the previous sibling.
    pub fn previous_sibling(&self) -> Option<&Rc<RefCell<VirtualEventNode>>> {
        self.previous_sibling.as_ref()
    }

    /// Get the next sibling.
    pub fn next_sibling(&self) -> Option<&Rc<RefCell<VirtualEventNode>>> {
        self.next_sibling.as_ref()
    }

    /// Replace a node with another.
    ///
    /// The new node is given the same siblings as the old node.
    pub fn replace_with_node(&mut self, mut new: VirtualEventNode) {
        new.previous_sibling = self.previous_sibling.take();
        new.next_sibling = self.next_sibling.take();

        *self = new;
    }

    /// Remove a child node from it's siblings.
    pub fn remove_node_from_siblings(&mut self, child: &Rc<RefCell<VirtualEventNode>>) {
        let mut child = child.borrow_mut();
        let is_first_sibling = child.previous_sibling.is_none();
        let is_last_sibling = child.next_sibling.is_none();

        let parent = self.as_element_mut().unwrap();
        if is_first_sibling && is_last_sibling {
            parent.children = None;
        } else if is_first_sibling {
            parent.children.as_mut().unwrap().first_child = child.next_sibling.clone().unwrap();
        } else if is_last_sibling {
            parent.children.as_mut().unwrap().last_child = child.previous_sibling.clone().unwrap();
        }

        match (
            child.previous_sibling.clone().as_mut(),
            child.next_sibling.as_mut(),
        ) {
            (Some(previous), Some(next)) => {
                previous.borrow_mut().next_sibling = Some(next.clone());
                next.borrow_mut().previous_sibling = Some(previous.clone());
            }
            (Some(previous), None) => {
                previous.borrow_mut().next_sibling = None;
            }
            (None, Some(next)) => {
                next.borrow_mut().previous_sibling = None;
            }
            (None, None) => {}
        };

        child.previous_sibling = None;
        child.next_sibling = None;
    }

    /// Insert a node before another node.
    pub fn insert_before(
        &mut self,
        new: Rc<RefCell<VirtualEventNode>>,
        existing: Rc<RefCell<VirtualEventNode>>,
    ) {
        let parent = self.as_element_mut().unwrap();

        {
            let mut new_borrow = new.borrow_mut();
            let mut existing_borrow = existing.borrow_mut();
            match existing_borrow.previous_sibling.take() {
                Some(previous) => {
                    previous.borrow_mut().next_sibling = Some(new.clone());
                    new_borrow.previous_sibling = Some(previous);
                }
                None => {
                    parent.children.as_mut().unwrap().first_child = new.clone();
                }
            };
        }

        new.borrow_mut().next_sibling = Some(existing.clone());
        existing.borrow_mut().previous_sibling = Some(new);
    }
}

impl VirtualEventElement {
    /// Create a new VirtualEventNode for the given events id.
    fn new(events_id: ElementEventsId) -> Self {
        VirtualEventElement {
            events_id,
            children: None,
        }
    }

    /// Get this node's unique id for its events.
    pub fn events_id(&self) -> ElementEventsId {
        self.events_id
    }

    /// Get the element's first child.
    pub fn first_child(&self) -> Option<Rc<RefCell<VirtualEventNode>>> {
        self.children.as_ref().map(|c| c.first_child.clone())
    }

    /// Append a child to the end of the list of children.
    pub fn append_child(&mut self, new_child: Rc<RefCell<VirtualEventNode>>) {
        match self.children.as_mut() {
            Some(children) => {
                {
                    children.last_child.borrow_mut().next_sibling = Some(new_child.clone());
                    let mut new_child_borrow = new_child.borrow_mut();

                    new_child_borrow.previous_sibling = Some(children.last_child.clone());
                    new_child_borrow.next_sibling = None;
                }

                children.last_child = new_child;
            }
            None => {
                self.set_first_and_last_child(new_child);
            }
        };
    }

    // Set this element's first and last child.
    fn set_first_and_last_child(&mut self, child: Rc<RefCell<VirtualEventNode>>) {
        self.children = Some(VirtualEventElementChildren {
            first_child: child.clone(),
            last_child: child.clone(),
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that we can append children to a virtual event node.
    #[test]
    fn append_children() {
        let events = VirtualEvents::new_with_prefix(1.);

        let mut node = events.create_element_node();
        let elem = node.as_element_mut().unwrap();

        let children = create_element_nodes(&events, 3);
        for child in &children {
            elem.append_child(child.clone());
        }

        assert_elem_children_equal(elem, &children);
    }

    /// Verify that we can insert nodes before another node in the virtual event nodes.
    #[test]
    fn insert_before() {
        let events = VirtualEvents::new_with_prefix(1.);

        let children = create_element_nodes(&events, 3);

        let mut node = events.create_element_node();

        {
            let elem = node.as_element_mut().unwrap();
            elem.append_child(children[0].clone());
        }

        node.insert_before(children[1].clone(), children[0].clone());
        node.insert_before(children[2].clone(), children[0].clone());

        let expected_order = [
            children[1].clone(),
            children[2].clone(),
            children[0].clone(),
        ];
        assert_elem_children_equal(node.as_element().unwrap(), &expected_order);
    }

    /// Verify that we can remove a node from its siblings.
    #[test]
    fn remove_node_from_siblings() {
        let events = VirtualEvents::new_with_prefix(1.);

        let children = create_element_nodes(&events, 3);

        let mut node = events.create_element_node();

        {
            let elem = node.as_element_mut().unwrap();
            for child in &children {
                elem.append_child(child.clone());
            }
        }

        node.remove_node_from_siblings(&children[1]);
        assert_elem_children_equal(
            node.as_element().unwrap(),
            &[children[0].clone(), children[2].clone()],
        );

        node.remove_node_from_siblings(&children[0]);
        assert_elem_children_equal(node.as_element().unwrap(), &[children[2].clone()]);

        node.remove_node_from_siblings(&children[2]);
        assert_elem_children_equal(node.as_element().unwrap(), &[]);
    }

    /// Verify that we can replace a node with another node.
    #[test]
    fn replace_node() {
        let events = VirtualEvents::new_with_prefix(1.);

        let children = create_element_nodes(&events, 3);

        let mut node = events.create_element_node();

        {
            let elem = node.as_element_mut().unwrap();
            for child in &children {
                elem.append_child(child.clone());
            }
        }

        let new_node = events.create_element_node();
        let new_node_events_id = new_node.as_element().unwrap().events_id;

        assert_eq!(node_events_id(&children[1]) == new_node_events_id, false);
        children[1].borrow_mut().replace_with_node(new_node);
        assert_eq!(node_events_id(&children[1]) == new_node_events_id, true);

        assert_elem_children_equal(
            node.as_element().unwrap(),
            &[
                children[0].clone(),
                children[1].clone(),
                children[2].clone(),
            ],
        );
    }

    fn create_element_nodes(
        events: &VirtualEvents,
        count: usize,
    ) -> Vec<Rc<RefCell<VirtualEventNode>>> {
        (0..count)
            .into_iter()
            .map(|_| {
                let child = events.create_element_node();
                let child = Rc::new(RefCell::new(child));
                child
            })
            .collect()
    }

    fn assert_elem_children_equal(
        elem: &VirtualEventElement,
        expected: &[Rc<RefCell<VirtualEventNode>>],
    ) {
        let mut idx = 0;

        let mut next_child = elem.first_child().clone();

        while let Some(child) = next_child {
            let child = child.borrow();

            if idx == 0 {
                assert_eq!(child.previous_sibling.is_none(), true);
            }

            assert_eq!(
                child.as_element().unwrap().events_id(),
                expected[idx].borrow().as_element().unwrap().events_id,
            );

            next_child = child.next_sibling.clone();
            idx += 1;

            if idx == expected.len() {
                assert_eq!(child.next_sibling.is_none(), true);
            }
        }

        assert_eq!(idx, expected.len());

        assert_elem_first_and_last_child(elem, expected);
    }

    fn assert_elem_first_and_last_child(
        elem: &VirtualEventElement,
        expected_children: &[Rc<RefCell<VirtualEventNode>>],
    ) {
        if expected_children.len() == 0 {
            assert!(elem.children.is_none());
            return;
        }

        let elem_children = elem.children.as_ref().unwrap();

        assert_eq!(
            node_events_id(&elem_children.first_child),
            node_events_id(expected_children.first().unwrap()),
        );

        assert_eq!(
            node_events_id(&elem_children.last_child),
            node_events_id(expected_children.last().unwrap()),
        );
    }

    fn node_events_id(node: &Rc<RefCell<VirtualEventNode>>) -> ElementEventsId {
        node.borrow().as_element().unwrap().events_id
    }
}
