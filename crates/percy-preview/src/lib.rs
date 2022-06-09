//! Preview view components.

#![deny(missing_docs)]

use std::cell::RefCell;
use std::rc::Rc;

use virtual_node::VirtualNode;

pub use self::rerender::Rerender;

mod rerender;

/// Describes a view component preview.
pub struct Preview {
    /// The name of this preview
    name: String,
    /// A description of the preview.
    /// This gets displayed above the preview.
    description: Option<String>,
    /// A url friendly version of the name.
    name_url_friendly: UrlFriendlyString,
    /// Render the preview
    renderer: Rc<RefCell<dyn FnMut() -> VirtualNode>>,
}

/// A string that only contains letters, numbers, hyphens and underscores.
struct UrlFriendlyString(String);

impl Preview {
    /// Create a new Preview.
    pub fn new<S: ToString>(name: S, render: Rc<RefCell<dyn FnMut() -> VirtualNode>>) -> Self {
        let name_url_friendly = UrlFriendlyString::new(name.to_string());
        Preview {
            name: name.to_string(),
            description: None,
            name_url_friendly,
            renderer: render,
        }
    }

    /// The name of the preview.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// A URL friendly version of the name.
    pub fn name_url_friendly(&self) -> &String {
        &self.name_url_friendly.0
    }

    /// The preview's description.
    pub fn description(&self) -> &Option<String> {
        &self.description
    }

    /// The preview's description.
    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }

    /// Returns a function that can be used to render the preview.
    pub fn renderer(&self) -> &Rc<RefCell<dyn FnMut() -> VirtualNode>> {
        &self.renderer
    }
}

impl UrlFriendlyString {
    /// Replaces non alphanumeric characters with hyphens.
    pub fn new<S: ToString>(string: S) -> Self {
        let string = string.to_string();
        let string = string.replace(" ", "-");

        let url_friendly_string: String = string
            .chars()
            .filter_map(|char| {
                if char == '-' || char == '_' {
                    return Some(char);
                }

                if !char.is_alphanumeric() {
                    return None;
                }

                return Some(char);
            })
            .collect();

        UrlFriendlyString(url_friendly_string.to_lowercase())
    }
}
