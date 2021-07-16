use std::cell::RefCell;

pub use self::event_attribute::*;
pub use self::input_event::*;
pub use self::mouse_event::*;

mod event_attribute;

mod input_event;
mod mouse_event;

/// An event handler that takes one argument.
pub type KnownEventHandler<T> = RefCell<Option<Box<dyn Fn(T)>>>;

/// TODO: Do we only need this in non-wasm testing environments? If so put this behind a test
///  utils flag or something.
pub struct KnownEvents {
    pub oninput: KnownEventHandler<DomInputEvent>,
    pub onclick: KnownEventHandler<DomMouseEvent>,
}

impl KnownEvents {
    pub fn new() -> Self {
        KnownEvents {
            oninput: RefCell::new(None),
            onclick: RefCell::new(None),
        }
    }
}

impl PartialEq for KnownEvents {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
