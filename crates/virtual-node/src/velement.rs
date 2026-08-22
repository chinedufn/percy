use crate::event::{Events, RealDom};
use crate::VirtualNode;
use std::collections::HashMap;
use std::fmt;

pub use self::attribute_value::*;
pub use self::special_attributes::*;

mod attribute_value;
mod special_attributes;

pub struct VirtualElement<Dom: RealDom> {
    /// The HTML tag, such as "div"
    pub tag: String,
    /// HTML attributes such as id, class, style, etc
    pub attrs: HashMap<String, AttributeValue>,
    /// Events that will get added to your real DOM element via `.addEventListener`
    ///
    /// Events natively handled in HTML such as onclick, onchange, oninput and others
    /// can be found in [`VElement.known_events`]
    pub events: Events<Dom>,
    /// The children of this `VirtualNode`. So a <div> <em></em> </div> structure would
    /// have a parent div and one child, em.
    pub children: Vec<VirtualNode<Dom>>,
    /// See [`SpecialAttributes`]
    pub special_attributes: SpecialAttributes<Dom>,
}

impl<Dom: RealDom> PartialEq for VirtualElement<Dom> {
    fn eq(&self, other: &Self) -> bool {
        let VirtualElement {
            tag: lhs_tag,
            attrs: lhs_attrs,
            events: lhs_events,
            children: lhs_children,
            special_attributes: lhs_special_attributes,
        } = self;
        let VirtualElement {
            tag: rhs_tag,
            attrs: rhs_attrs,
            events: rhs_events,
            children: rhs_children,
            special_attributes: rhs_special_attributes,
        } = other;

        lhs_tag == rhs_tag
            && lhs_attrs == rhs_attrs
            && lhs_events == rhs_events
            && lhs_children == rhs_children
            && lhs_special_attributes == rhs_special_attributes
    }
}

impl<Dom: RealDom> VirtualElement<Dom> {
    pub fn new(tag: impl Into<String>) -> VirtualElement<Dom> {
        VirtualElement {
            tag: tag.into(),
            attrs: HashMap::new(),
            events: Events::new(),
            children: vec![],
            special_attributes: SpecialAttributes::default(),
        }
    }
}

impl<Dom: RealDom> fmt::Debug for VirtualElement<Dom> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Element(<{}>, attrs: {:?}, children: {:?})",
            self.tag, self.attrs, self.children,
        )
    }
}

impl<Dom: RealDom> fmt::Display for VirtualElement<Dom> {
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
