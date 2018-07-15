use std::collections::HashMap;
use std::fmt;
pub use std::cell::RefCell;
pub use std::rc::Rc;
use webapis::*;

#[derive(PartialEq)]
pub struct VirtualNode {
    pub tag: String,
    pub props: HashMap<String, String>,
    pub events: Events,
    pub children: Vec<Rc<RefCell<VirtualNode>>>,
    /// We keep track of parents during the `html!` macro in order to be able to crawl
    /// up the tree and assign newly found nodes to the proper parent.
    /// By the time an `html!` macro finishes all nodes will have `parent` None
    pub parent: Option<Rc<RefCell<VirtualNode>>>,
    /// Some(String) if this is a [text node](https://developer.mozilla.org/en-US/docs/Web/API/Text).
    /// When patching these into a real DOM these use `document.createTextNode(text)`
    pub text: Option<String>,
}

impl VirtualNode {
    /// Create a new virtual node with a given tag.
    ///
    /// These get patched into the DOM using `document.createElement`
    ///
    /// ```
    /// let div = VirtualNode::tag("div");
    /// ```
    pub fn new (tag: &str) -> VirtualNode {
        let props = HashMap::new();
        let events = Events(HashMap::new());
        VirtualNode {
            tag: tag.to_string(),
            props,
            events,
            children: vec![],
            parent: None,
            text: None
        }
    }

    /// Create a text node.
    ///
    /// These get patched into the DOM using `document.createTextNode`
    ///
    /// ```
    /// let div = VirtualNode::text("div");
    /// ```
    pub fn text (text: &str) -> VirtualNode {
        VirtualNode {
            tag: "".to_string(),
            props: HashMap::new(),
            events: Events(HashMap::new()),
            children: vec![],
            parent: None,
            text: Some(text.to_string())
        }
    }
}

impl VirtualNode {
    /// Build a DOM element by recursively creating DOM nodes for this element and it's
    /// children, it's children's children, etc.
    pub fn create_element(&self) -> Element {
        let elem = document.create_element(&self.tag);

        self.children.iter().for_each(|child| {
            let child = child.borrow();
            elem.append_child(child.create_element())
        });

        elem
    }
}

impl<'a> From<&'a str> for VirtualNode {
    // Used by our html! macro to turn "Strings of text" into virtual nodes.
    fn from(text: &'a str) -> Self {
        VirtualNode::text(text)
    }
}

impl fmt::Debug for VirtualNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VirtualNode | tag: {}, props: {:#?}, text: {:#?}, children: {:#?} |", self.tag, self.props, self.text, self.children)
    }
}

/// We need a custom implementation of fmt::Debug since Fn() doesn't
/// implement debug.
pub struct Events(pub HashMap<String, Box<Fn() -> ()>>);

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
