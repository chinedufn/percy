use crate::event::{EventHandler, EventName, MouseEvent, VirtualEvents, ELEMENT_EVENTS_ID_PROP};
use crate::{Closure, PercyDom};
use js_sys::Reflect;
use virtual_node::event::ElementEventsId;
use wasm_bindgen::JsCast;

impl PercyDom {
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
            // `dyn_into().unwrap()` was crashing in Firefox (but not Chrome) when running a
            //  real-world application, even though our click event integration tests are passing
            //  in Firefox.
            //  This was observed in `web-sys 0.3.61`
            let target_element: web_sys::Element = target.unchecked_into();

            bubble_event(target_element, MouseEvent::new(event), &events);
        };
        let callback = Box::new(callback) as Box<dyn FnMut(_)>;
        let callback = Closure::wrap(callback);

        self.root_node
            .add_event_listener_with_callback(event, callback.as_ref().unchecked_ref())
            .unwrap();

        self.event_delegation_listeners
            .insert(event, Box::new(callback));
    }
}

// Call the event, then call it on its parent, etc
fn bubble_event(elem: web_sys::Element, mouse_event: MouseEvent, events: &VirtualEvents) {
    let events_id = Reflect::get(&elem, &ELEMENT_EVENTS_ID_PROP.into()).unwrap();
    let events_id = events_id.as_string();

    if let Some(events_id) = events_id {
        let events_id = events_id.trim_start_matches(&events.events_id_props_prefix().to_string());
        let events_id: u32 = events_id.parse().unwrap();
        let events_id = ElementEventsId::new(events_id);

        let cb = events.get_event_handler(&events_id, &EventName::ONCLICK);

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
