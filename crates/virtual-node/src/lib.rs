//! The virtual_node module exposes the `VirtualNode` struct and methods that power our
//! virtual dom.

// TODO: A few of these dependencies (including js_sys) are used to power events.. yet events
// only work on wasm32 targets. So we should start sprinkling some
//
// #[cfg(target_arch = "wasm32")]
// #[cfg(not(target_arch = "wasm32"))]
//
// Around in order to get rid of dependencies that we don't need in non wasm32 targets

use std::fmt;

use crate::event::{VirtualEventNode, VirtualEvents};
use web_sys::{self, Node};

pub use self::create_element::VIRTUAL_NODE_MARKER_PROPERTY;
pub use self::event::EventAttribFn;
pub use self::iterable_nodes::*;
pub use self::velement::*;
pub use self::vtext::*;

pub mod event;
pub mod test_utils;

mod create_element;

mod iterable_nodes;
mod velement;
mod vtext;

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

impl VirtualNode {
    /// Create a new virtual element node with a given tag.
    ///
    /// These get patched into the DOM using `document.createElement`
    ///
    /// ```
    /// # use virtual_node::VirtualNode;
    /// let _div = VirtualNode::element("div");
    /// ```
    // FIXME: Rename to new_element
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
    /// ```
    /// # use virtual_node::VirtualNode;
    /// let _text = VirtualNode::text("My text node");
    /// ```
    // FIXME: Rename to new_text
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
    // TODO: Rename to .as_velement()
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
    // TODO: Rename to .as_vtext()
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

    /// Create and return a [`web_sys::Node`] along with its events.
    pub fn create_dom_node(&self, events: &mut VirtualEvents) -> (Node, VirtualEventNode) {
        match self {
            VirtualNode::Text(text_node) => {
                (text_node.create_text_node().into(), VirtualEventNode::Text)
            }
            VirtualNode::Element(element_node) => {
                let (elem, events) = element_node.create_element_node(events);
                (elem.into(), VirtualEventNode::Element(events))
            }
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

    /// Verify that a boolean attribute is included in the string if true.
    #[test]
    fn boolean_attribute_true_shown() {
        let mut button = VElement::new("button");
        button.attrs.insert("disabled".into(), true.into());

        let expected = "<button disabled></button>";
        let button = VirtualNode::Element(button).to_string();

        assert_eq!(button.to_string(), expected);
    }

    /// Verify that a boolean attribute is not included in the string if false.
    #[test]
    fn boolean_attribute_false_ignored() {
        let mut button = VElement::new("button");
        button.attrs.insert("disabled".into(), false.into());

        let expected = "<button></button>";
        let button = VirtualNode::Element(button).to_string();

        assert_eq!(button.to_string(), expected);
    }
}
