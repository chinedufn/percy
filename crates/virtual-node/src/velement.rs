use std::collections::HashMap;
use std::fmt;

use crate::event::Events;
use crate::VirtualNode;

#[derive(PartialEq)]
pub struct VElement {
    /// The HTML tag, such as "div"
    pub tag: String,
    /// HTML attributes such as id, class, style, etc
    pub attrs: HashMap<String, String>,
    /// Events that will get added to your real DOM element via `.addEventListener`
    ///
    /// Events natively handled in HTML such as onclick, onchange, oninput and others
    /// can be found in [`VElement.known_events`]
    pub custom_events: Events,
    /// The children of this `VirtualNode`. So a <div> <em></em> </div> structure would
    /// have a parent div and one child, em.
    pub children: Vec<VirtualNode>,
}

impl VElement {
    pub fn new<S>(tag: S) -> Self
    where
        S: Into<String>,
    {
        VElement {
            tag: tag.into(),
            attrs: HashMap::new(),
            custom_events: Events(HashMap::new()),
            children: vec![],
        }
    }
}

impl fmt::Debug for VElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Element(<{}>, attrs: {:?}, children: {:?})",
            self.tag, self.attrs, self.children,
        )
    }
}

impl fmt::Display for VElement {
    // Turn a VElement and all of it's children (recursively) into an HTML string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}", self.tag).unwrap();

        for (attr, value) in self.attrs.iter() {
            write!(f, r#" {}="{}""#, attr, value)?;
        }

        write!(f, ">")?;

        for child in self.children.iter() {
            write!(f, "{}", child.to_string())?;
        }

        if !html_validation::is_self_closing(&self.tag) {
            write!(f, "</{}>", self.tag)?;
        }

        Ok(())
    }
}
