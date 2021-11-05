use std::collections::HashMap;
use std::fmt;

use crate::event::Events;
use crate::VirtualNode;

pub use self::attribute_value::*;
pub use self::special_attributes::*;

mod attribute_value;
mod special_attributes;

#[derive(PartialEq)]
pub struct VElement {
    /// The HTML tag, such as "div"
    pub tag: String,
    /// HTML attributes such as id, class, style, etc
    pub attrs: HashMap<String, AttributeValue>,
    /// Events that will get added to your real DOM element via `.addEventListener`
    ///
    /// Events natively handled in HTML such as onclick, onchange, oninput and others
    /// can be found in [`VElement.known_events`]
    pub events: Events,
    /// The children of this `VirtualNode`. So a <div> <em></em> </div> structure would
    /// have a parent div and one child, em.
    pub children: Vec<VirtualNode>,
    /// See [`SpecialAttributes`]
    pub special_attributes: SpecialAttributes,
}

impl VElement {
    pub fn new<S>(tag: S) -> Self
    where
        S: Into<String>,
    {
        VElement {
            tag: tag.into(),
            attrs: HashMap::new(),
            events: Events::new(),
            children: vec![],
            special_attributes: SpecialAttributes::default(),
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
            match value {
                AttributeValue::String(value_str) => {
                    write!(f, r#" {}="{}""#, attr, value_str)?;
                }
                AttributeValue::Bool(value_bool) => {
                    if *value_bool {
                        write!(f, " {}", attr)?;
                    }
                }
            }
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
