use crate::event::{
    ELEMENT_EVENTS_ID_PROP, EventHandler, EventName, MouseEventWebSys, VirtualEvents,
};
use crate::{Closure, PercyDom};
use js_sys::Reflect;
use std::ops::Deref;
use virtual_node::event::ElementEventsId;
use wasm_bindgen::{JsCast, JsValue};

impl PercyDom {
    /// Attach all of the event listeners that handle event delegation.
    ///
    /// See [`virtual_node::VirtualElement::<web_sys::Window>::add_events`] for where delegated
    /// events get added to virtual elements.
    ///
    /// Non-delegated events get handled by [`virtual_node::event::insert_non_delegated_event`].
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

            bubble_event(target_element, MouseEventWebSys::new(event), &events);
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
fn bubble_event(
    elem: web_sys::Element,
    mouse_event: MouseEventWebSys,
    events: &VirtualEvents<web_sys::Window>,
) {
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
                EventHandler::Custom(func) => {
                    // This branch can get called if a user creates an `EventHandler::Custom`
                    //  for a mouse event such as `onmouseclick`. This is because the
                    //  `VirtualEvents::add_events` calls `EventName::is_delegated`, and
                    //  `EventName::is_delegated` returns `true` if the event is `onclick`.

                    let func: &js_sys::Function = func.as_ref().as_ref().unchecked_ref();
                    func.call1(&JsValue::NULL, &mouse_event.deref().clone())
                        .unwrap();
                }
            };
        }
    }

    if !mouse_event.should_propagate().get() {
        return;
    }

    if let Some(parent) = elem.parent_element() {
        bubble_event(parent, mouse_event, events);
    }
}
