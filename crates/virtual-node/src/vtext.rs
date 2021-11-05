use std::fmt;

use web_sys::Text;

/// Represents a text node
#[derive(PartialEq)]
pub struct VText {
    pub text: String,
}

impl VText {
    /// Create an new `VText` instance with the specified text.
    pub fn new<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        VText { text: text.into() }
    }

    /// Return a `Text` element from a `VirtualNode`, typically right before adding it
    /// into the DOM.
    pub(crate) fn create_text_node(&self) -> Text {
        let document = web_sys::window().unwrap().document().unwrap();
        document.create_text_node(&self.text)
    }
}

impl From<&str> for VText {
    fn from(text: &str) -> Self {
        VText {
            text: text.to_string(),
        }
    }
}

impl From<String> for VText {
    fn from(text: String) -> Self {
        VText { text }
    }
}

impl fmt::Debug for VText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Text({})", self.text)
    }
}

// Turn a VText into an HTML string
impl fmt::Display for VText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}
