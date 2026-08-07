use std::fmt;

/// Represents a text node
#[derive(PartialEq)]
pub struct VirtualText {
    pub text: String,
}

impl VirtualText {
    /// Create an new `VText` instance with the specified text.
    pub fn new<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        VirtualText { text: text.into() }
    }

    /// Return a `Text` element from a `VirtualNode`, typically right before adding it
    /// into the DOM.
    #[cfg(feature = "web")]
    pub(crate) fn create_text_node(&self) -> web_sys::Text {
        use crate::create_element::set_virtual_node_marker;

        let document = web_sys::window().unwrap().document().unwrap();
        let text = document.create_text_node(&self.text);

        set_virtual_node_marker(&text);

        text
    }
}

impl From<&str> for VirtualText {
    fn from(text: &str) -> Self {
        VirtualText {
            text: text.to_string(),
        }
    }
}

impl From<String> for VirtualText {
    fn from(text: String) -> Self {
        VirtualText { text }
    }
}

impl fmt::Debug for VirtualText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Text({})", self.text)
    }
}

// Turn a VText into an HTML string
impl fmt::Display for VirtualText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}
