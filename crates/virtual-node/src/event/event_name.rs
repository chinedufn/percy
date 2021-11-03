use std::borrow::Cow;

/// The name of the event with the `on` prefix.
///
/// onclick, oninput, onmousemove ... etc
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct EventName(Cow<'static, str>);

impl EventName {
    /// Create a new `EventName`.
    ///
    /// # Panics
    ///
    /// Panics if the event does not start with "on". Such as "onclick".
    pub fn new(name: Cow<'static, str>) -> Self {
        EventName(name)
    }

    /// `onclick`, `onmousemove` etc
    pub fn with_on_prefix(&self) -> &str {
        &self.0.as_ref()
    }

    /// `click`, `mousemove` etc
    pub fn without_on_prefix(&self) -> &str {
        &self.0.as_ref()[2..]
    }
}

impl EventName {
    /// Whether or not this event gets handled by our event delegation system.
    /// If not, the event will be attached to the DOM element.
    pub fn is_delegated(&self) -> bool {
        if self == &Self::ONCLICK {
            true
        } else {
            false
        }
    }
}

impl EventName {
    /// The "onclick" event
    pub const ONCLICK: EventName = EventName(Cow::Borrowed("onclick"));

    /// The "oninput" event
    pub const ONINPUT: EventName = EventName(Cow::Borrowed("oninput"));
}

impl From<&'static str> for EventName {
    fn from(event_name: &'static str) -> Self {
        EventName(Cow::Borrowed(event_name))
    }
}

impl From<String> for EventName {
    fn from(event_name: String) -> Self {
        EventName(Cow::Owned(event_name))
    }
}
