use crate::VElement;

use crate::event::{insert_non_delegated_event, set_events_id, ElementEventsId, VirtualEvents};
use web_sys::Element;

use crate::event::ELEMENT_EVENTS_ID_PROP;
use js_sys::Reflect;

impl VElement {
    pub(super) fn add_events(
        &self,
        element: &Element,
        events: &VirtualEvents,
        events_id: ElementEventsId,
    ) {
        set_events_id(element, events, events_id);
        Reflect::set(
            element,
            &ELEMENT_EVENTS_ID_PROP.into(),
            &format!("{}{}", events.events_id_props_prefix(), events_id.get()).into(),
        )
        .unwrap();

        let needs_create_closures = self.events.has_events();
        if needs_create_closures {
            for (onevent, callback) in self.events.events() {
                if onevent.is_delegated() {
                    events.insert_event(events_id, onevent.clone(), callback.clone(), None);
                } else {
                    insert_non_delegated_event(element, onevent, callback, events_id, events);
                }
            }
        }
    }
}
