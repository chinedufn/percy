//! The virtual_node module exposes the `VirtualNode` struct and methods that power our
//! virtual dom.

// TODO: A few of thse dependencies (including js_sys) are used to power events.. yet events
// only work on wasm32 targest. So we should start sprinkling some
//
// #[cfg(target_arch = "wasm32")]
// #[cfg(not(target_arch = "wasm32"))]
//
// Around in order to get rid of dependencies that we don't need in non wasm32 targets


pub use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
pub use std::rc::Rc;

pub mod virtual_node_test_utils;

use web_sys;
use web_sys::*;

use js_sys::Function;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

mod browser_events;
pub use self::browser_events::*;

/// When building your views you'll typically use the `html!` macro to generate
/// `VirtualNode`'s.
///
/// `html! { <div> <span></span> </div> }` really generates a `VirtualNode` with
/// one child (span).
///
/// Later, on the client side, you'll use the `diff` and `patch` modules to
/// update the real DOM with your latest tree of virtual nodes (virtual dom).
///
/// Or on the server side you'll just call `.to_string()` on your root virtual node
/// in order to recursively render the node and all of its children.
///
/// TODO: Make all of these fields private and create accessor methods
#[derive(PartialEq)]
pub struct VirtualNode {
    /// The HTML tag, such as "div"
    pub tag: String,
    /// HTML props such as id, class, style, etc
    pub props: HashMap<String, String>,
    /// Events that will get added to your real DOM element via `.addEventListener`
    pub custom_events: CustomEvents,
    /// Events that are specified in web_sys such as `oninput` `onclick` `onmousemove`
    /// etc...
    pub browser_events: BrowserEvents,
    /// The children of this `VirtualNode`. So a <div> <em></em> </div> structure would
    /// have a parent div and one child, em.
    pub children: Option<Vec<VirtualNode>>,
    /// Some(String) if this is a [text node](https://developer.mozilla.org/en-US/docs/Web/API/Text).
    /// When patching these into a real DOM these use `document.createTextNode(text)`
    pub text: Option<String>,
}

/// Our html! macro takes in tokens, builds `ParsedVirtualNode`'s from those tokens and then
/// finally converts that `ParsedVirtualNode` into a `VirtualNode`.
///
/// When we next revisit that macro we'll want to revisit whether or not we can build a `VirtualNode`
/// as we go vs. needing this intermediary data structure.
///
/// TODO: Is this complexity really necessary? Doubt it... Map this all out on paper... shouldn't need
/// two nearly identical structs...?
#[derive(PartialEq)]
pub struct ParsedVirtualNode {
    /// TODO: See if we can get rid of ParsedVirtualNode entirely in favor of only VirtualNode
    pub tag: String,
    /// TODO: See if we can get rid of ParsedVirtualNode entirely in favor of only VirtualNode
    pub props: HashMap<String, String>,
    /// TODO: See if we can get rid of ParsedVirtualNode entirely in favor of only VirtualNode
    pub custom_events: CustomEvents,
    /// TODO: See if we can get rid of ParsedVirtualNode entirely in favor of only VirtualNode
    pub browser_events: BrowserEvents,
    /// TODO: See if we can get rid of ParsedVirtualNode entirely in favor of only VirtualNode
    /// TODO: Don't think this needs to be an option
    pub children: Option<Vec<Rc<RefCell<ParsedVirtualNode>>>>,
    /// TODO: See if we can get rid of ParsedVirtualNode entirely in favor of only VirtualNode
    pub parent: Option<Rc<RefCell<ParsedVirtualNode>>>,
    /// TODO: See if we can get rid of ParsedVirtualNode entirely in favor of only VirtualNode
    pub text: Option<String>,
}

impl ParsedVirtualNode {
    /// Create a virtual node that is meant to represent a DOM element
    pub fn new(tag: &str) -> ParsedVirtualNode {
        let props = HashMap::new();
        let events = CustomEvents(HashMap::new());
        ParsedVirtualNode {
            tag: tag.to_string(),
            props,
            custom_events: events,
            browser_events: BrowserEvents::default(),
            children: Some(vec![]),
            parent: None,
            text: None,
        }
    }

    /// Create a virtual node that is meant to represent DOM Text
    pub fn text(text: &str) -> ParsedVirtualNode {
        ParsedVirtualNode {
            tag: "".to_string(),
            props: HashMap::new(),
            browser_events: BrowserEvents::default(),
            custom_events: CustomEvents(HashMap::new()),
            children: Some(vec![]),
            parent: None,
            text: Some(text.to_string()),
        }
    }

    /// Take off the the `VirtualNode`'s direct descendants (who in turn might have their
    /// own descendants)
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
            browser_events: parsed_node.browser_events,
            custom_events: parsed_node.custom_events,
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
        let custom_events = CustomEvents(HashMap::new());
        let browser_events = BrowserEvents::default();
        VirtualNode {
            tag: tag.to_string(),
            props,
            custom_events,
            browser_events,
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
            custom_events: CustomEvents(HashMap::new()),
            browser_events: BrowserEvents::default(),
            children: Some(vec![]),
            text: Some(text.to_string()),
        }
    }
}

impl VirtualNode {
    /// Build a DOM element by recursively creating DOM nodes for this element and it's
    /// children, it's children's children, etc.
    pub fn create_element(&self) -> Element {
        let document = web_sys::window().unwrap().document().unwrap();

        let current_elem = document.create_element(&self.tag).unwrap();

        self.props.iter().for_each(|(name, value)| {
            current_elem
                .set_attribute(name, value)
                .expect("Set element attribute in create element");
        });

        self.custom_events.0.iter().for_each(|(onevent, callback)| {
            // onclick -> click
            let event = &onevent[2..];

            let mut callback = callback.borrow_mut();
            let callback = callback.take().unwrap();
            (current_elem.as_ref() as &web_sys::EventTarget)
                .add_event_listener_with_callback(event, &callback.as_ref().unchecked_ref())
                .unwrap();

            callback.forget();
        });

        // Handle the `oninput` event
        let mut callback = self.browser_events.oninput.borrow_mut().take();
        if callback.is_some() {
            let callback = callback.unwrap();
            (current_elem.as_ref() as &web_sys::EventTarget)
                .add_event_listener_with_callback("input", &callback.as_ref().unchecked_ref())
                .unwrap();

            callback.forget();
        }

        let mut previous_node_was_text = false;

        self.children.as_ref().unwrap().iter().for_each(|child| {
            if child.is_text_node() {
                let current_node = current_elem.as_ref() as &web_sys::Node;

                // We ensure that the text siblings are patched by preventing the browser from merging
                // neighboring text nodes. Originally inspired by some of React's work from 2016.
                //  -> https://reactjs.org/blog/2016/04/07/react-v15.html#major-changes
                //  -> https://github.com/facebook/react/pull/5753
                //
                // `ptns` = Percy text node separator
                if previous_node_was_text {
                    let separator = document.create_comment("ptns");

                    current_node
                        .append_child(separator.as_ref() as &web_sys::Node)
                        .unwrap();
                }

                current_node
                    .append_child(
                        document
                            .create_text_node(&child.text.as_ref().unwrap())
                            .as_ref() as &web_sys::Node,
                    )
                    .unwrap();

                previous_node_was_text = true;
            } else {
                previous_node_was_text = false;

                (current_elem.as_ref() as &web_sys::Node)
                    .append_child(child.create_element().as_ref() as &web_sys::Node)
                    .unwrap();
            }
        });

        current_elem
    }

    /// Return a `Text` element from a `VirtualNode`, typically right before adding it
    /// into the DOM.
    pub fn create_text_node(&self) -> Text {
        let document = web_sys::window().unwrap().document().unwrap();
        document.create_text_node(&self.text.as_ref().unwrap())
    }

    /// Whether or not this `VirtualNode` is representing a `Text` node
    pub fn is_text_node(&self) -> bool {
        self.text.is_some()
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
            browser_events: node.browser_events,
            custom_events: node.custom_events,
            children,
            parent: None,
            text: node.text,
        }
    }
}
impl From<Vec<VirtualNode>> for ParsedVirtualNode {
    fn from(nodes: Vec<VirtualNode>) -> Self {
        let parsed_nodes: Vec<Rc<RefCell<ParsedVirtualNode>>> = nodes
            .into_iter()
            .map(|node| Rc::new(RefCell::new(ParsedVirtualNode::from(node))))
            .collect();

        let mut wrapper = ParsedVirtualNode::new("__VEC_OF_CHILDREN__");
        wrapper.children = Some(parsed_nodes);

        wrapper
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
        if self.is_text_node() {
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


/// We need a custom implementation of fmt::Debug since FnMut() doesn't
/// implement debug.
pub struct CustomEvents(pub HashMap<String, RefCell<Option<Closure<FnMut(Event) -> ()>>>>);

impl PartialEq for CustomEvents {
    // TODO: What should happen here..? And why?
    fn eq(&self, _rhs: &Self) -> bool {
        true
    }
}

impl fmt::Debug for CustomEvents {
    // Print out all of the event names for this VirtualNode
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let events: String = self.0.keys().map(|key| " ".to_string() + key).collect();
        write!(f, "{}", events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Move this somewhere that we can use the `html!` macro
//    #[test]
//    fn to_string() {
//        let node = html! {
//        <div id="some-id", !onclick=|_ev| {},>
//            <span>
//                { "Hello world" }
//            </span>
//        </div>
//        };
//        let expected = r#"<div id="some-id"><span>Hello world</span></div>"#;
//
//        assert_eq!(node.to_string(), expected);
//    }
}
