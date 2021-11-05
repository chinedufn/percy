use crate::VElement;

use crate::event::{insert_non_delegated_event, EventsByNodeIdx, ManagedEvent};
use web_sys::Element;

use crate::event::EVENTS_ID_PROP;
use js_sys::Reflect;

impl VElement {
    pub(super) fn add_events(&self, element: &Element, events: &EventsByNodeIdx, node_idx: u32) {
        let needs_create_closures = self.events.has_events();
        if needs_create_closures {
            for (onevent, callback) in self.events.events() {
                let events_clone = events.clone();

                Reflect::set(
                    element,
                    &EVENTS_ID_PROP.into(),
                    &format!("{}{}", events_clone.events_id_props_prefix(), node_idx).into(),
                )
                .unwrap();

                if onevent.is_delegated() {
                    events.insert_managed_event(
                        node_idx,
                        onevent.clone(),
                        ManagedEvent::Delegated(callback.clone()),
                    );
                } else {
                    insert_non_delegated_event(element, onevent, callback, node_idx, events);
                }
            }
        }
    }
}
