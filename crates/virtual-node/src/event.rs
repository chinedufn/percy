use std::collections::HashMap;
use std::fmt;
use std::ops::{Deref, DerefMut};

/// Box<dyn AsRef<JsValue>>> is our js_sys::Closure. Stored this way to allow us to store
/// any Closure regardless of the types of its arguments.
pub type EventAttribFn = std::rc::Rc<dyn AsRef<wasm_bindgen::JsValue>>;

/// We need a custom implementation of fmt::Debug since JsValue doesn't
/// implement debug.
pub struct Events(pub HashMap<String, EventAttribFn>);

impl PartialEq for Events {
    // TODO: What should happen here..? And why?
    fn eq(&self, _rhs: &Self) -> bool {
        true
    }
}

impl fmt::Debug for Events {
    // Print out all of the event names for this VirtualNode
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let events: String = self.0.keys().map(|key| " ".to_string() + key).collect();
        write!(f, "{}", events)
    }
}

impl Deref for Events {
    type Target = HashMap<String, EventAttribFn>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Events {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
