//! The virtual_node module exposes the `VirtualNode` struct and methods that power our
//! virtual dom.

// TODO: A few of these dependencies (including js_sys) are used to power events.. yet events
// only work on wasm32 targest. So we should start sprinkling some
//
// #[cfg(target_arch = "wasm32")]
// #[cfg(not(target_arch = "wasm32"))]
//
// Around in order to get rid of dependencies that we don't need in non wasm32 targets

use std::collections::{HashSet,HashMap};
use std::fmt;
use std::rc::Rc;

pub mod virtual_node_test_utils;

use web_sys::{self, Text, Element, Node, EventTarget};

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use lazy_static::lazy_static;

use std::ops::Deref;
use std::sync::Mutex;


// Used to uniquely identify elements that contain closures so that the DomUpdater can
// look them up by their unique id.
// When the DomUpdater sees that the element no longer exists it will drop all of it's
// Rc'd Closures for those events.
lazy_static! {
    static ref ELEM_UNIQUE_ID: Mutex<u32> = Mutex::new(0);

    static ref SELF_CLOSING_TAGS: HashSet<&'static str> = [
        "area", "base", "br", "col", "hr", "img", "input", "link", "meta",
        "param", "command", "keygen", "source",
    ].iter().cloned().collect();
}

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
/// TODO: Create a builder to create instances of VirtualNode::Element with
/// props and children without having to explicitly create a VElement
#[derive(PartialEq)]
pub enum VirtualNode {
    /// An element node (node type `ELEMENT_NODE`).
    Element(VElement),
    /// A text node (node type `TEXT_NODE`).
    ///
    /// Note: This wraps a `VText` instead of a plain `String` in
    /// order to enable custom methods like `create_text_node()` on the
    /// wrapped type.
    Text(VText),
}

#[derive(PartialEq)]
pub struct VElement {
    /// The HTML tag, such as "div"
    pub tag: String,
    /// HTML props such as id, class, style, etc
    pub props: HashMap<String, String>,
    /// Events that will get added to your real DOM element via `.addEventListener`
    pub events: Events,
    /// The children of this `VirtualNode`. So a <div> <em></em> </div> structure would
    /// have a parent div and one child, em.
    pub children: Vec<VirtualNode>,
}

#[derive(PartialEq)]
pub struct VText {
    pub text: String,
}

impl VirtualNode {
    /// Create a new virtual element node with a given tag.
    ///
    /// These get patched into the DOM using `document.createElement`
    ///
    /// ```ignore
    /// use virtual_dom_rs::VirtualNode;
    ///
    /// let div = VirtualNode::element("div");
    /// ```
    pub fn element<S>(tag: S) -> Self where S: Into<String> {
        VirtualNode::Element(VElement::new(tag))
    }

    /// Create a new virtual text node with the given text.
    ///
    /// These get patched into the DOM using `document.createTextNode`
    ///
    /// ```ignore
    /// use virtual_dom_rs::VirtualNode;
    ///
    /// let div = VirtualNode::text("div");
    /// ```
    pub fn text<S>(text: S) -> Self where S: Into<String> {
        VirtualNode::Text(VElement::new(text.into()))
    }

    /// Return a [`VElement`] reference, if this is an [`Element`] variant.
    ///
    /// [`VElement`]: struct.VElement.html
    /// [`Element`]: enum.VirtualNode.html#variant.Element
    pub fn as_velement_ref(&self) -> Option<&VElement> {
        match self {
            VirtualNode::Element(ref element_node) => Some(element_node),
            _ => None,
        }
    }

    /// Return a mutable [`VElement`] reference, if this is an [`Element`] variant.
    ///
    /// [`VElement`]: struct.VElement.html
    /// [`Element`]: enum.VirtualNode.html#variant.Element
    pub fn as_velement_mut(&mut self) -> Option<&mut VElement> {
        match self {
            VirtualNode::Element(ref mut element_node) => Some(element_node),
            _ => None,
        }
    }

    /// Return a [`VText`] reference, if this is an [`Text`] variant.
    ///
    /// [`VText`]: struct.VText.html
    /// [`Text`]: enum.VirtualNode.html#variant.Text
    pub fn as_vtext_ref(&self) -> Option<&VText> {
        match self {
            VirtualNode::Text(ref text_node) => Some(text_node),
            _ => None,
        }
    }

    /// Return a mutable [`VText`] reference, if this is an [`Text`] variant.
    ///
    /// [`VText`]: struct.VText.html
    /// [`Text`]: enum.VirtualNode.html#variant.Text
    pub fn as_vtext_mut(&mut self) -> Option<&mut VText> {
        match self {
            VirtualNode::Text(ref mut text_node) => Some(text_node),
            _ => None,
        }
    }

    /// Create and return a `CreatedNode` instance (containing a DOM `Node`
    /// together with potentially related closures) for this virtual node.
    pub fn create_dom_node(&self) -> CreatedNode<Node> {
        match self {
            VirtualNode::Text(text_node) => CreatedNode::without_closures(text_node.create_text_node()),
            VirtualNode::Element(element_node) => element_node.create_element_node().into(),
        }
    }
}

impl VElement {
    pub fn new<S>(tag: S) -> Self where S: Into<String> {
        VElement {
            tag: tag.into(),
            props: HashMap::new(),
            events: Events(HashMap::new()),
            children: vec![],
        }
    }

    /// Whether or not this is a self closing tag such as <br> or <img />
    pub fn is_self_closing(&self) -> bool {
        SELF_CLOSING_TAGS.contains(self.tag.as_str())
    }

    /// Build a DOM element by recursively creating DOM nodes for this element and it's
    /// children, it's children's children, etc.
    pub fn create_element_node(&self) -> CreatedNode<Element> {
        let document = web_sys::window().unwrap().document().unwrap();

        let element = document.create_element(&self.tag).unwrap();
        let mut closures = HashMap::new();

        self.props.iter().for_each(|(name, value)| {
            element
                .set_attribute(name, value)
                .expect("Set element attribute in create element");
        });

        if self.events.0.len() > 0 {
            let unique_id = create_unique_identifier();

            element
                .set_attribute("data-vdom-id".into(), &unique_id.to_string())
                .expect("Could not set attribute on element");

            closures.insert(unique_id, vec![]);

            self.events.0.iter().for_each(|(onevent, callback)| {
                // onclick -> click
                let event = &onevent[2..];

                let current_elem: &EventTarget = element.dyn_ref().unwrap();

                current_elem
                    .add_event_listener_with_callback(
                        event,
                        callback.as_ref().as_ref().unchecked_ref(),
                    )
                    .unwrap();

                closures
                    .get_mut(&unique_id)
                    .unwrap()
                    .push(Rc::clone(callback));
            });
        }

        let mut previous_node_was_text = false;

        self.children.iter().for_each(|child| {
            match child {
                VirtualNode::Text(text_node) => {
                    let current_node = element.as_ref() as &web_sys::Node;

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
                        .append_child(&text_node.create_text_node())
                        .unwrap();

                    previous_node_was_text = true;
                },
                VirtualNode::Element(element_node) => {
                    previous_node_was_text = false;

                    let child = element_node.create_element_node();
                    let child_elem: Element = child.node;

                    closures.extend(child.closures);

                    element.append_child(&child_elem).unwrap();
                },
            }
        });

        CreatedNode { node: element, closures }
    }

}

impl VText {
    /// Create an new `VText` instance with the specified text.
    pub fn new<S>(text: S) -> Self where S: Into<String> {
        VText { text: text.into() }
    }

    /// Return a `Text` element from a `VirtualNode`, typically right before adding it
    /// into the DOM.
    pub fn create_text_node(&self) -> Text {
        let document = web_sys::window().unwrap().document().unwrap();
        document.create_text_node(&self.text)
    }
}

/// A node along with all of the closures that were created for that
/// node's events and all of it's child node's events.
pub struct CreatedNode<T> {
    /// A `Node` or `Element` that was created from a `VirtualNode`
    pub node: T,
    /// A map of a node's unique identifier along with all of the Closures for that node.
    ///
    /// The DomUpdater uses this to look up nodes and see if they're still in the page. If not
    /// the reference that we maintain to their closure will be dropped, thus freeing the Closure's
    /// memory.
    pub closures: HashMap<u32, Vec<DynClosure>>,
}

impl<T> CreatedNode<T> {
    pub fn without_closures<N: Into<T>>(node: N) -> Self {
        CreatedNode {
            node: node.into(),
            closures: HashMap::with_capacity(0),
        }
    }
}

impl<T> Deref for CreatedNode<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl From<CreatedNode<Element>> for CreatedNode<Node> {
    fn from(other: CreatedNode<Element>) -> CreatedNode<Node> {
        CreatedNode {
            node: other.node.into(),
            closures: other.closures,
        }
    }
}

fn create_unique_identifier() -> u32 {
    let mut elem_unique_id = ELEM_UNIQUE_ID.lock().unwrap();

    *elem_unique_id += 1;

    *elem_unique_id
}

impl From<VText> for VirtualNode {
    fn from(other: VText) -> Self {
        VirtualNode::Text(other)
    }
}

impl From<VElement> for VirtualNode {
    fn from(other: VElement) -> Self {
        VirtualNode::Element(other)
    }
}

impl From<&str> for VText {
    fn from(text: &str) -> Self {
        VText { text: text.to_string() }
    }
}

impl From<String> for VText {
    fn from(text: String) -> Self {
        VText { text }
    }
}

impl IntoIterator for VirtualNode {
    type Item = VirtualNode;
    // TODO: Is this possible with an array [VirtualNode] instead of a vec?
    type IntoIter = ::std::vec::IntoIter<VirtualNode>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self].into_iter()
    }
}

impl fmt::Debug for VirtualNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VirtualNode::Element(e) => write!(f, "Node::{:?}", e),
            VirtualNode::Text(t) => write!(f, "Node::{:?}", t),
        }
    }
}

impl fmt::Debug for VElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Element(<{}>, props: {:?}, children: {:?})",
            self.tag, self.props, self.children,
        )
    }
}

impl fmt::Debug for VText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Text({})", self.text)
    }
}

impl fmt::Display for VElement {
    // Turn a VElement and all of it's children (recursively) into an HTML string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}", self.tag).unwrap();

        for (prop, value) in self.props.iter() {
            write!(f, r#" {}="{}""#, prop, value)?;
        }

        write!(f, ">")?;

        for child in self.children.iter() {
            write!(f, "{}", child.to_string())?;
        }

        if !self.is_self_closing() {
            write!(f, "</{}>", self.tag)?;
        }

        Ok(())
    }
}

// Turn a VText into an HTML string
impl fmt::Display for VText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

// Turn a VirtualNode into an HTML string (delegate impl to variants)
impl fmt::Display for VirtualNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VirtualNode::Element(element) => write!(f, "{}", element),
            VirtualNode::Text(text) => write!(f, "{}", text),
        }
    }
}

/// Box<dyn AsRef<JsValue>>> is our js_sys::Closure. Stored this way to allow us to store
/// any Closure regardless of the arguments.
pub type DynClosure = Rc<dyn AsRef<JsValue>>;

/// We need a custom implementation of fmt::Debug since JsValue doesn't
/// implement debug.
pub struct Events(pub HashMap<String, DynClosure>);

impl PartialEq for Events {
    // TODO: What should happen here..? And why?
    fn eq(&self, _rhs: &Self) -> bool {
        true
    }
}

impl fmt::Debug for Events {
    // Print out all of the event names for this VirtualNode
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let events: String = self.0.keys().map(|key| " ".to_string() + key).collect();
        write!(f, "{}", events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_closing_tag_to_string() {
        let node = VirtualNode::element("br");

        // No </br> since self closing tag
        assert_eq!(&node.to_string(), "<br>");
    }

    // TODO: Use html_macro as dev dependency and uncomment
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
