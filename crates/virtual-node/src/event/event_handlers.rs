use crate::event::RealDom;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

/// Event handlers such as the closure in `onclick = |mouse_event| {}`.
pub enum EventHandler<Dom: RealDom> {
    /// A callback that does not contain any arguments.
    NoArgs(Rc<RefCell<dyn FnMut()>>),
    /// Handle mouse events such as `onclick` and `oninput`
    MouseEvent(Rc<RefCell<dyn FnMut(Dom::MouseEvent)>>),
    /// EventHandler's that we do not have a dedicated type for.
    /// This is useful for custom events.
    Custom(Dom::EventCallback),
}
impl<Dom: RealDom> Clone for EventHandler<Dom> {
    fn clone(&self) -> Self {
        match self {
            Self::NoArgs(func) => Self::NoArgs(func.clone()),
            Self::MouseEvent(func) => Self::MouseEvent(func.clone()),
            Self::Custom(func) => Self::Custom(func.clone()),
        }
    }
}

/// A mouse event.
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent)
#[derive(Clone)]
#[cfg(feature = "web")]
pub struct MouseEventWebSys {
    event: web_sys::MouseEvent,
    should_propagate: Rc<std::cell::Cell<bool>>,
}

#[cfg(feature = "web")]
impl MouseEventWebSys {
    /// Create a new MouseEvent.
    pub fn new(event: web_sys::MouseEvent) -> Self {
        MouseEventWebSys {
            event,
            should_propagate: Rc::new(std::cell::Cell::new(true)),
        }
    }

    /// Prevent the event from propagating.
    pub fn stop_propagation(&self) {
        self.should_propagate.set(false);
        self.event.stop_propagation();
    }

    /// Whether or not the event should propagate.
    pub fn should_propagate(&self) -> &Rc<std::cell::Cell<bool>> {
        &self.should_propagate
    }
}

#[cfg(feature = "web")]
impl std::ops::Deref for MouseEventWebSys {
    type Target = web_sys::MouseEvent;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

// Allows us to easily derive PartialEq for some of the types that contain events.
// Those PartialEq implementations are used for testing.
// Maybe we can put some of the event related PartialEq implementations
// behind a #[cfg(any(test, feature = "__test-utils"))].
impl<Dom: RealDom> PartialEq for EventHandler<Dom> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<Dom: RealDom> Debug for EventHandler<Dom> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("event handler")
    }
}
