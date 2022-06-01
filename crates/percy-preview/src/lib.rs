//! Preview view components.

#![deny(missing_docs)]

use virtual_node::VirtualNode;

/// Allows the preview to trigger a rerender of itself.
///
/// For example, a preview for a component with a button that increments
/// a counter might trigger a rerender whenever the button is clicked.
pub type Rerender = std::rc::Rc<dyn FnMut() -> ()>;

/// Describes a view component preview.
pub struct Preview {
    /// The name of this preview
    name: UrlSafeString,
    /// Render the preview
    render: Box<dyn FnMut(Rerender) -> VirtualNode>,
}

/// A string that only contains letters, numbers, hyphens and underscores.
pub struct UrlSafeString(String);

impl Preview {
    /// Create a new Preview.
    pub fn new(name: UrlSafeString, render: Box<dyn FnMut(Rerender) -> VirtualNode>) -> Self {
        Preview { name, render }
    }

    /// The name of the preview.
    pub fn name(&self) -> &String {
        &self.name.0
    }
}

impl UrlSafeString {
    /// If the String that contains only letters, numbers, hyphens and underscores,
    /// we return the `UrlSafeString`.
    /// Otherwise we return `None`.
    pub fn new(string: String) -> Option<Self> {
        let all_chars_valid = string
            .chars()
            .all(|char| char.is_alphanumeric() || char == '-' || char == '_');

        if all_chars_valid {
            Some(UrlSafeString(string))
        } else {
            None
        }
    }
}
