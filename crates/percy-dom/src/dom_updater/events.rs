use crate::event::{EventHandler, EventName, EventsByNodeIdx, MouseEvent, EVENTS_ID_PROP};
use crate::{Closure, DomUpdater};
use js_sys::Reflect;
use wasm_bindgen::JsCast;

impl DomUpdater {
    /// Attach all of the event listeners that handle event delegation.
    pub(super) fn attach_event_listeners(&mut self) {
        self.attach_onclick_listener();
    }

    fn attach_onclick_listener(&mut self) {
        let event = "click";
        debug_assert!(EventName::new(format!("on{}", event).into()).is_delegated());

        let events = self.events.clone();

        let callback = move |event: web_sys::MouseEvent| {
            let target = event.target().unwrap();

            bubble_event(target.dyn_into().unwrap(), MouseEvent::new(event), &events);
        };
        let callback = Box::new(callback) as Box<dyn FnMut(_)>;
        let callback = Closure::wrap(callback);

        self.root_node
            .add_event_listener_with_callback(event, callback.as_ref().as_ref().unchecked_ref())
            .unwrap();

        self.event_delegation_listeners
            .insert(event, Box::new(callback));
    }
}

// Call the event, then call it on its parent, etc
fn bubble_event(elem: web_sys::Element, mouse_event: MouseEvent, events: &EventsByNodeIdx) {
    let events_id = Reflect::get(&elem, &EVENTS_ID_PROP.into()).unwrap();
    let events_id = events_id.as_string();

    if let Some(events_id) = events_id {
        let events_id = events_id.trim_start_matches(&events.events_id_props_prefix().to_string());
        let node_id: u32 = events_id.parse().unwrap();

        let cb = events.get_event_handler(&node_id, &EventName::ONCLICK);

        if let Some(cb) = cb {
            match cb {
                EventHandler::NoArgs(no_args) => (no_args.borrow_mut())(),
                EventHandler::MouseEvent(mouse) => {
                    (mouse.borrow_mut())(mouse_event.clone());
                }
                _ => panic!(),
            };
        }
    }

    if !mouse_event.should_propagate().get() {
        return;
    }

    if let Some(parent) = elem.parent_node() {
        if let Ok(parent) = parent.dyn_into() {
            bubble_event(parent, mouse_event, events);
        }
    }
}

// When an element is clicked, the global click handler needs to be able to call a handler based
// on that particular element.
//
// So, the global handler must be able to get a unique identifier from the element, and use that
// event ID to look up the element's callback.
//
// So, elements need to be marked in the DOM with some sort of identifier.
//
// When an element is removed from the DOM we want to remove its corresponding event handlers.
// Ideally we can do this without needing to read any information from the DOM.
//
// So, ideally IDs can be gotten or derived from the virtual DOM.
//
// If id's were based on the node idx within the tree, then we could simply diff/patch the presence
// of events.
//
// If a node has events, add the node idx to .nodeIdx property on the element in the DOM.
// If the node loses its events, remove the property from the DOM.
//
// No matter what, update the closures stored in the EventHandlers type for every node that has
// closures.
// Which boils down to clearing the HashMap then inserting each of the events.
//
// So, when we diff we need to keep track of nodeIdx's that have events that did not have any,
// nodeIdx's that no longer have events that used to have, and nodeidx's that previously had events
// and still currently have events.
//
// # Cases
// - If went from some to none,
//    - remove .nodeIdx from DOM node
//    - remove closures memory store
//
// - If went from none -> some
//    - add .nodeIdx to DOM node
//    - insert closures into memory store
//
// - If went from some -> some
//    - insert closures into memory store
//
// - If creating a new node,
//   - iterate depth first through its children and call the none -> some case
//      for all of the nodes using their .nodeIdx (increment by one as we traverse)
//
// # Implementation
//
// - [DONE] Make `.create_dom_node()` take a `node_idx: u32` property
//
// - [DONE] Make `VirtualElement.create_dom_element` take a `node_idx: u32` property.
//
// - [DONE] Create Patch::RemoveClosureIdx(NodeIdx) that removes .__nodeIdx from the DOM node
//
// - [DONE] Create Patch::SetClosureIdx(NodeIdx, u32) that sets .__nodeIdx = u32 on the DOM node
//
// - [DONE] Add diff tests for creating RemoveClosureIdxFromDomNode if it is a delegated event
//
// - [DONE] Add diff tests for creating SetClosureIdx if it is a delegated event
//
// - [DONE] Add diff tests for removing event listeners
//
// - [DONE] Add diff tests for adding event listeners
//
// - [DONE] Add diff test verifying that removed event listener patches occur before add event
//       listener patches.
//
// - [DONE] Refactor `event_listener_closures.rs` tests to be DRY.
//
// - [DONE] Add wasm-bindgen-test's to `event_listener_closures.rs` for
//     - [DONE] Rename file to `events`
//     - [DONE] Applying RemoveClosureIdx patch
//     - [DONE] Applying SetClosureIdx patch
//     - [DONE] Setting .__nodeIdx on newly created nodes that have events
//     - [DONE] Setting .__nodeIdx on the children of newly created nodes
//
// - [DONE] Change EventsByNodeIdx to hold an enum { Delegated, NotDelegated } instead of an Option
//
// - [DONE] diff test: if an event is removed from a node with events, push a patch to remove the
//       EventsByNodeIdx
//
// - [DONE] diff test, push patch to remove all tracked events for a node when we
//   - [DONE] Replace it
//   - [DONE] Replaced its ancestor
//   - [DONE] Truncate it (during truncate children)
//
// - [DONE] During diff, is an event is added to a node, push a patch to add the EventsByNodeIdx
//
// - [DONE] During diff, if an event is removed, push a Patch::RemoveEvent(NodeIdx, &'a EventName);
//
// - [DONE] Add `wasm-bindgen-test` to events.rs for clicking a child element and still triggering the
//       parent element's onclick.
//
// - [DONE] Add `wasm-bindgen-test` to events.rs for stop propagation (click child element that has
//        `onclick` handler but parent does not trigger if stop_propagation() is called
//
// - [DONE] Fill out the unimplemented!() tests in events.rs
//
// - [DONE] Remove all of the ActiveClosures logic
//
// - [DONE] Remove `.vdom-id` in `add_events.rs`
//
// - [DONE] Create `DomUpdator.attach_event_listeners()` which attaches listener for click handler.
//
// - [DONE] Get the test suite passing
//
// - [DONE] Add a test where we add a non delegated event, remove it, then add it again and verify that
//       the text is modified only by the final event.
//
// - [DONE] Merge
//
// - [DONE] In the Patch::Replace, add an `.old_node_idx` and `.new_node_idx`
//
// - [DONE] In the `Patch::AppendChildren` add `.old_node_idx` and `.new_node_idx`
//
// - [DONE] Add diff test that we generate the correct new_node_idx when replacing a node.
//
// - [DONE] Add diff test that we generate the correct new_node_idx when appending a child node
//       Create first node with no children, second node with two children, and verify that the
//       children's new nodes indices are 1 and 2
//
// - [DONE] Add patch test to events.rs that we properly set the events ID on a replaced node using the new
//       node idx (not the old)
//
// - [DONE] Add patch test to events.rs that we properly set the events id on an appended child using the
//       new node Idx (not the old).
//
// - [DONE] Add patch test for replacing a text node with an element and that element has correct ID
//
// - [DONE] Add internal design docs on events
//
// - [ ] In new commit make `OnCreateElem` take a `Cow<'static, str>` instead of a u32.
//
// - [ ] Rename DomUpdater to PercyDom
//
// - [ ] Use the afia dashboard and make sure events work as expected.
//
// - [ ] minor version bump for percy-dom and virtual-node
