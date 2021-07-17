use crate::{EventAttribFn, VElement};

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;
use wasm_bindgen::JsCast;
use web_sys::Element;

lazy_static! {
    static ref ELEM_UNIQUE_ID: Mutex<u32> = Mutex::new(0);
}

fn create_unique_identifier() -> u32 {
    let mut elem_unique_id = ELEM_UNIQUE_ID.lock().unwrap();
    *elem_unique_id += 1;
    *elem_unique_id
}

impl VElement {
    pub(super) fn add_events(
        &self,
        element: &Element,
        closures: &mut HashMap<u32, Vec<EventAttribFn>>,
    ) {
        let needs_create_closures = self.custom_events.0.len() > 0;

        if needs_create_closures {
            let unique_id = create_unique_identifier();

            element
                .set_attribute("data-vdom-id".into(), &unique_id.to_string())
                .expect("Could not set attribute on element");

            closures.insert(unique_id, vec![]);

            #[cfg(target_arch = "wasm32")]
            {
                self.custom_events.0.iter().for_each(|(onevent, callback)| {
                    // onclick -> click
                    let event_name = &onevent[2..];

                    attach_event(&element, event_name, callback, closures, unique_id);
                });
            }
        }
    }
}

// event_name is the name without the 'on' prefix
//   -> "input" ... "click" ... "change" ... etc
#[cfg(target_arch = "wasm32")]
fn attach_event(
    element: &web_sys::Element,
    event_name: &str,
    callback: &EventAttribFn,
    closures: &mut HashMap<u32, Vec<EventAttribFn>>,
    unique_id: u32,
) {
    let current_elem: &web_sys::EventTarget = element.dyn_ref().unwrap();

    current_elem
        .add_event_listener_with_callback(event_name, callback.as_ref().as_ref().unchecked_ref())
        .unwrap();

    closures
        .get_mut(&unique_id)
        .unwrap()
        .push(Rc::clone(callback));
}
