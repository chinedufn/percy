//! The virtual_node module exposes the `VirtualNode` struct and methods that power our
//! virtual dom.

// TODO: A few of these dependencies (including js_sys) are used to power events.. yet events
// only work on wasm32 targest. So we should start sprinkling some
//
// #[cfg(target_arch = "wasm32")]
// #[cfg(not(target_arch = "wasm32"))]
//
// Around in order to get rid of dependencies that we don't need in non wasm32 targets

use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
use std::sync::Mutex;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
use web_sys::{self, Element, EventTarget, Node, Text};

// Used to uniquely identify elements that contain closures so that the DomUpdater can
// look them up by their unique id.
// When the DomUpdater sees that the element no longer exists it will drop all of it's
// Rc'd Closures for those events.
use crate::event::Events;

pub use self::event::EventAttribFn;

pub mod event;
pub mod test_utils;

mod create_element;

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
/// attrs and children without having to explicitly create a VElement
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
    /// # use percy_dom::VirtualNode;
    /// let div = VirtualNode::element("div");
    /// ```
    pub fn element<S>(tag: S) -> Self
    where
        S: Into<String>,
    {
        VirtualNode::Element(VElement::new(tag))
    }

    /// Create a new virtual text node with the given text.
    ///
    /// These get patched into the DOM using `document.createTextNode`
    ///
    /// ```ignore
    /// # use percy_dom::VirtualNode;
    /// let div = VirtualNode::text("div");
    /// ```
    pub fn text<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        VirtualNode::Text(VText::new(text.into()))
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
            VirtualNode::Text(text_node) => {
                CreatedNode::without_closures(text_node.create_text_node())
            }
            VirtualNode::Element(element_node) => element_node.create_element_node().into(),
        }
    }

    /// Used by html-macro to insert space before text that is inside of a block that came after
    /// an open tag.
    ///
    /// html! { <div> {world}</div> }
    ///
    /// So that we end up with <div> world</div> when we're finished parsing.
    pub fn insert_space_before_text(&mut self) {
        match self {
            VirtualNode::Text(text_node) => {
                text_node.text = " ".to_string() + &text_node.text;
            }
            _ => {}
        }
    }

    /// Used by html-macro to insert space after braced text if we know that the next block is
    /// another block or a closing tag.
    ///
    /// html! { <div>{Hello} {world}</div> } -> <div>Hello world</div>
    /// html! { <div>{Hello} </div> } -> <div>Hello </div>
    ///
    /// So that we end up with <div>Hello world</div> when we're finished parsing.
    pub fn insert_space_after_text(&mut self) {
        match self {
            VirtualNode::Text(text_node) => {
                text_node.text += " ";
            }
            _ => {}
        }
    }
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

impl VText {
    /// Create an new `VText` instance with the specified text.
    pub fn new<S>(text: S) -> Self
    where
        S: Into<String>,
    {
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
    pub closures: HashMap<u32, Vec<EventAttribFn>>,
}

impl<T> CreatedNode<T> {
    pub fn without_closures<N: Into<T>>(node: N) -> Self {
        CreatedNode {
            node: node.into(),
            closures: std::collections::HashMap::with_capacity(0),
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

/// A trait with common functionality for rendering front-end views.
pub trait View {
    /// Render a VirtualNode, or any IntoIter<VirtualNode>
    fn render(&self) -> VirtualNode;
}

impl<V> From<&V> for VirtualNode
where
    V: View,
{
    fn from(v: &V) -> Self {
        v.render()
    }
}

/// Used by the html! macro for all braced child nodes so that we can use any type
/// that implements Into<IterableNodes>
///
/// html! { <div> { nodes } </div> }
///
/// nodes can be a String .. VirtualNode .. Vec<VirtualNode> ... etc
pub struct IterableNodes(Vec<VirtualNode>);

impl IterableNodes {
    /// Retrieve the first node mutably
    pub fn first(&mut self) -> &mut VirtualNode {
        self.0.first_mut().unwrap()
    }

    /// Retrieve the last node mutably
    pub fn last(&mut self) -> &mut VirtualNode {
        self.0.last_mut().unwrap()
    }
}

impl IntoIterator for IterableNodes {
    type Item = VirtualNode;
    // TODO: Is this possible with an array [VirtualNode] instead of a vec?
    type IntoIter = ::std::vec::IntoIter<VirtualNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<VirtualNode> for IterableNodes {
    fn from(other: VirtualNode) -> Self {
        IterableNodes(vec![other])
    }
}

impl From<&str> for IterableNodes {
    fn from(other: &str) -> Self {
        IterableNodes(vec![VirtualNode::text(other)])
    }
}

impl From<String> for IterableNodes {
    fn from(other: String) -> Self {
        IterableNodes(vec![VirtualNode::text(other.as_str())])
    }
}

impl From<Vec<VirtualNode>> for IterableNodes {
    fn from(other: Vec<VirtualNode>) -> Self {
        IterableNodes(other)
    }
}

impl<V: View> From<Vec<V>> for IterableNodes {
    fn from(other: Vec<V>) -> Self {
        IterableNodes(other.into_iter().map(|it| it.render()).collect())
    }
}

impl<V: View> From<&Vec<V>> for IterableNodes {
    fn from(other: &Vec<V>) -> Self {
        IterableNodes(other.iter().map(|it| it.render()).collect())
    }
}

impl<V: View> From<&[V]> for IterableNodes {
    fn from(other: &[V]) -> Self {
        IterableNodes(other.iter().map(|it| it.render()).collect())
    }
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

impl From<&str> for VirtualNode {
    fn from(other: &str) -> Self {
        VirtualNode::text(other)
    }
}

impl From<String> for VirtualNode {
    fn from(other: String) -> Self {
        VirtualNode::text(other.as_str())
    }
}

impl From<&str> for VText {
    fn from(text: &str) -> Self {
        VText {
            text: text.to_string(),
        }
    }
}

impl From<String> for VText {
    fn from(text: String) -> Self {
        VText { text }
    }
}

impl IntoIterator for VirtualNode {
    type Item = VirtualNode;
    // TODO: ::std::iter::Once<VirtualNode> to avoid allocation
    type IntoIter = ::std::vec::IntoIter<VirtualNode>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self].into_iter()
    }
}

impl Into<::std::vec::IntoIter<VirtualNode>> for VirtualNode {
    fn into(self) -> ::std::vec::IntoIter<VirtualNode> {
        self.into_iter()
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
            "Element(<{}>, attrs: {:?}, children: {:?})",
            self.tag, self.attrs, self.children,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_closing_tag_to_string() {
        let node = VirtualNode::element("br");

        // No </br> since self closing tag
        assert_eq!(&node.to_string(), "<br>");
    }

    #[test]
    fn to_string() {
        let mut node = VirtualNode::Element(VElement::new("div"));
        node.as_velement_mut()
            .unwrap()
            .attrs
            .insert("id".into(), "some-id".into());

        let mut child = VirtualNode::Element(VElement::new("span"));

        let text = VirtualNode::Text(VText::new("Hello world"));

        child.as_velement_mut().unwrap().children.push(text);

        node.as_velement_mut().unwrap().children.push(child);

        let expected = r#"<div id="some-id"><span>Hello world</span></div>"#;

        assert_eq!(node.to_string(), expected);
    }
}
