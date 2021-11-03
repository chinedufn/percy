use crate::EventAttribFn;
use std::cell::{Cell, RefCell};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::rc::Rc;

/// Event handlers such as the closure in `onclick = |event| {}`.
///
/// ## Cloning
///
/// Can be cheaply cloned since since inner types are reference counted.
#[derive(Clone)]
pub enum EventHandler {
    /// A callback that does not contain any arguments.
    NoArgs(Rc<RefCell<dyn FnMut()>>),
    /// Handle mouse events such as `onclick` and `oninput`
    MouseEvent(Rc<RefCell<dyn FnMut(MouseEvent)>>),
    /// EventHandler's that we do not have a dedicated type for.
    /// This is useful for custom events.
    UnsupportedSignature(EventAttribFn),
}

/// A mouse event.
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent)
#[derive(Clone)]
pub struct MouseEvent {
    event: web_sys::MouseEvent,
    should_propagate: Rc<Cell<bool>>,
}

impl MouseEvent {
    /// Create a new MouseEvent.
    pub fn new(event: web_sys::MouseEvent) -> Self {
        MouseEvent {
            event,
            should_propagate: Rc::new(Cell::new(true)),
        }
    }

    /// Prevent the event from propagating.
    pub fn stop_propagation(&self) {
        self.should_propagate.set(false);
        self.event.stop_propagation();
    }

    /// Whether or not the event should propagate.
    pub fn should_propagate(&self) -> &Rc<Cell<bool>> {
        &self.should_propagate
    }
}

impl Deref for MouseEvent {
    type Target = web_sys::MouseEvent;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

// Allows us to easily derive PartialEq for some of the types that contain events.
// Those PartialEq implementations are used for testing.
// Maybe we can put some of the event related PartialEq implementations
// behind a #[cfg(any(test, feature = "__test-utils"))].
impl PartialEq for EventHandler {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Debug for EventHandler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("event handler")
    }
}
