pub use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
pub use std::rc::Rc;
use wasm_bindgen::prelude::Closure;
use webapis::*;

#[derive(PartialEq)]
pub struct VirtualNode {
    pub tag: String,
    pub props: HashMap<String, String>,
    pub events: Events,
    pub children: Option<Vec<VirtualNode>>,
    /// Some(String) if this is a [text node](https://developer.mozilla.org/en-US/docs/Web/API/Text).
    /// When patching these into a real DOM these use `document.createTextNode(text)`
    pub text: Option<String>,
}

// TODO: Is this complexity really necessary? Doubt it... Map this all out on paper... shouldn't need
// two nearly identical structs
#[derive(PartialEq)]
pub struct ParsedVirtualNode {
    pub tag: String,
    pub props: HashMap<String, String>,
    pub events: Events,
    // TODO: Don't think this needs to be an option
    pub children: Option<Vec<Rc<RefCell<ParsedVirtualNode>>>>,
    pub parent: Option<Rc<RefCell<ParsedVirtualNode>>>,
    pub text: Option<String>,
}

impl ParsedVirtualNode {
    pub fn new(tag: &str) -> ParsedVirtualNode {
        let props = HashMap::new();
        let events = Events(HashMap::new());
        ParsedVirtualNode {
            tag: tag.to_string(),
            props,
            events,
            children: Some(vec![]),
            parent: None,
            text: None,
        }
    }

    pub fn text(text: &str) -> ParsedVirtualNode {
        ParsedVirtualNode {
            tag: "".to_string(),
            props: HashMap::new(),
            events: Events(HashMap::new()),
            children: Some(vec![]),
            parent: None,
            text: Some(text.to_string()),
        }
    }

    pub fn take_children(&mut self) -> Vec<VirtualNode> {
        self.children
            .take()
            .unwrap()
            .into_iter()
            .map(|child| VirtualNode::from(Rc::try_unwrap(child).unwrap().into_inner()))
            .collect()
    }
}

impl From<ParsedVirtualNode> for VirtualNode {
    fn from(mut parsed_node: ParsedVirtualNode) -> Self {
        let children = Some(parsed_node.take_children());
        VirtualNode {
            tag: parsed_node.tag,
            props: parsed_node.props,
            events: parsed_node.events,
            children,
            text: parsed_node.text,
        }
    }
}

impl VirtualNode {
    /// Create a new virtual node with a given tag.
    ///
    /// These get patched into the DOM using `document.createElement`
    ///
    /// ```
    /// use virtual_dom_rs::VirtualNode;
    ///
    /// let div = VirtualNode::new("div");
    /// ```
    pub fn new(tag: &str) -> VirtualNode {
        let props = HashMap::new();
        let events = Events(HashMap::new());
        VirtualNode {
            tag: tag.to_string(),
            props,
            events,
            children: Some(vec![]),
            text: None,
        }
    }

    /// Create a text node.
    ///
    /// These get patched into the DOM using `document.createTextNode`
    ///
    /// ```
    /// use virtual_dom_rs::VirtualNode;
    ///
    /// let div = VirtualNode::text("div");
    /// ```
    pub fn text(text: &str) -> VirtualNode {
        VirtualNode {
            tag: "".to_string(),
            props: HashMap::new(),
            events: Events(HashMap::new()),
            children: Some(vec![]),
            text: Some(text.to_string()),
        }
    }
}

impl VirtualNode {
    /// Build a DOM element by recursively creating DOM nodes for this element and it's
    /// children, it's children's children, etc.
    pub fn create_element(&mut self) -> Element {
        let elem = document.create_element(&self.tag);

        self.props.iter().for_each(|(name, value)| {
            elem.set_attribute(name, value);
        });

        self.events.0.iter_mut().for_each(|(onevent, callback)| {
            // onclick -> click
            let event = &onevent[2..];

            let callback = callback.take().unwrap();
            elem.add_event_listener(event, &callback);
            callback.forget();
        });

        self.children
            .as_mut()
            .unwrap()
            .iter_mut()
            .for_each(|child| {
                if child.text.is_some() {
                    elem.append_text_child(
                        document.create_text_node(&child.text.as_ref().unwrap()),
                    );
                }

                if child.text.is_none() {
                    elem.append_child(child.create_element());
                }
            });

        elem
    }
}

// Used by our html! macro to turn "Strings of text" into virtual nodes.
impl<'a> From<&'a str> for ParsedVirtualNode {
    fn from(text: &'a str) -> Self {
        ParsedVirtualNode::text(text)
    }
}
impl From<String> for ParsedVirtualNode {
    fn from(text: String) -> Self {
        ParsedVirtualNode::text(&text)
    }
}
impl<'a> From<&'a String> for ParsedVirtualNode {
    fn from(text: &'a String) -> Self {
        ParsedVirtualNode::text(text)
    }
}
impl From<VirtualNode> for ParsedVirtualNode {
    fn from(mut node: VirtualNode) -> Self {
        let children = Some(node.wrap_children());

        ParsedVirtualNode {
            tag: node.tag,
            props: node.props,
            events: node.events,
            children,
            parent: None,
            text: node.text,
        }
    }
}

impl VirtualNode {
    fn wrap_children(&mut self) -> Vec<Rc<RefCell<ParsedVirtualNode>>> {
        self.children
            .take()
            .unwrap()
            .into_iter()
            .map(|child| wrap(child))
            .collect()
    }
}

fn wrap(v: VirtualNode) -> Rc<RefCell<ParsedVirtualNode>> {
    Rc::new(RefCell::new(ParsedVirtualNode::from(v)))
}

impl fmt::Debug for VirtualNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "VirtualNode | tag: {}, props: {:#?}, text: {:#?}, children: {:#?} |",
            self.tag, self.props, self.text, self.children
        )
    }
}

impl fmt::Debug for ParsedVirtualNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "VirtualNode | tag: {}, props: {:#?}, text: {:#?}, children: {:#?} |",
            self.tag, self.props, self.text, self.children
        )
    }
}

impl fmt::Display for VirtualNode {
    // Turn a VirtualNode and all of it's children (recursively) into an HTML string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.text.is_some() {
            write!(f, "{}", self.text.as_ref().unwrap())
        } else {
            write!(f, "<{}", self.tag).unwrap();

            for (prop, value) in self.props.iter() {
                write!(f, r#" {}="{}""#, prop, value)?;
            }

            write!(f, ">");

            for child in self.children.as_ref().unwrap().iter() {
                write!(f, "{}", child.to_string())?;
            }
            write!(f, "</{}>", self.tag)
        }
    }
}

/// We need a custom implementation of fmt::Debug since Fn() doesn't
/// implement debug.
pub struct Events(pub HashMap<String, Option<Closure<Fn() -> ()>>>);

impl PartialEq for Events {
    // TODO: What should happen here..? And why?
    fn eq(&self, _rhs: &Self) -> bool {
        true
    }
}

impl fmt::Debug for Events {
    // Print out all of the event names for this VirtualNode
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let events: String = self.0.keys().map(|key| format!("{} ", key)).collect();
        write!(f, "{}", events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        let node = html! {
        <div id="some-id", !onclick=|| {},>
            <span>
                { "Hello world" }
            </span>
        </div>
        };
        let expected = r#"<div id="some-id"><span>Hello world</span></div>"#;

        assert_eq!(node.to_string(), expected);
    }
}
