//!

use std::collections::HashMap;
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

mod html_macro;

#[derive(PartialEq)]
pub struct VirtualNode {
    tag: String,
    props: HashMap<String, String>,
    events: Events,
    children: Vec<Rc<RefCell<VirtualNode>>>,
    /// We keep track of parents during the `html!` macro in order to be able to crawl
    /// up the tree and assign newly found nodes to the proper parent.
    /// By the time an `html!` macro finishes all nodes will have `parent` None
    parent: Option<Rc<RefCell<VirtualNode>>>,
    /// Some(String) if this is a [text node](https://developer.mozilla.org/en-US/docs/Web/API/Text).
    /// When patching these into a real DOM these use `document.createTextNode(text)`
    text: Option<String>,
}

impl<'a> From<&'a str> for VirtualNode {
    fn from(text: &'a str) -> Self {
        VirtualNode::text(text)
    }
}

impl fmt::Debug for VirtualNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VirtualNode | tag: {}, props: {:#?}, text: {:#?}, children: {:#?} |", self.tag, self.props, self.text, self.children)
    }
}

// TODO: No longer need this since we implement partialeq ourselves for VirtualNode
pub struct Events(HashMap<String, Box<Fn() -> ()>>);

impl PartialEq for Events {
    // Once you set events on an element you can't change them, so we don't factor them
    // into our PartialEq
    fn eq(&self, rhs: &Self) -> bool {
       true
    }
}

impl fmt::Debug for Events {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let events: String = self.0.keys().map(|key| format!("{} ", key)).collect();
        write!(f, "{}", events)
    }
}

impl VirtualNode {
    fn new (tag: &str) -> VirtualNode {
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

    fn text (text: &str) -> VirtualNode {
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

