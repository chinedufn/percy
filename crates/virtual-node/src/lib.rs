//! The virtual_node module exposes the `VirtualNode` struct and methods that power our
//! virtual dom.

#[cfg(feature = "web")]
pub use self::create_element::VIRTUAL_NODE_MARKER_PROPERTY;
#[cfg(feature = "web")]
pub use self::event::EventAttribFn;
pub use self::iterable_nodes::*;
pub use self::velement::*;
pub use self::vtext::*;
use crate::event::{EventHandler, RealDom};
use std::fmt;

pub mod event;
pub mod test_utils;

#[cfg(feature = "web")]
mod create_element;

mod iterable_nodes;
mod velement;
mod vtext;

/// A [`VirtualNode`] whose [`RealDom`] is a [`web_sys::Window`].
#[cfg(feature = "web")]
pub type VirtualNodeWebSys = VirtualNode<web_sys::Window>;

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
/// ## Examples
/// ```
/// use virtual_node::VirtualNode;
/// let div = VirtualNode::<()>::new_element("div");
/// assert_eq!(div.to_string(), "<div></div>");
/// ```
pub enum VirtualNode<Dom: RealDom> {
    /// An element node (node type `ELEMENT_NODE`).
    Element(VirtualElement<Dom>),
    /// A text node (node type `TEXT_NODE`).
    ///
    /// Note: This wraps a `VText` instead of a plain `String` in
    /// order to enable custom methods like `create_text_node()` on the
    /// wrapped type.
    Text(VirtualText),
}

impl<Dom: RealDom> PartialEq for VirtualNode<Dom> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Element(lhs), Self::Element(rhs)) => lhs == rhs,
            (Self::Text(lhs), Self::Text(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

impl<Dom: RealDom> VirtualNode<Dom> {
    /// Create a new virtual element node with a given tag.
    ///
    /// These get patched into the DOM using `document.createElement`
    ///
    /// ```
    /// # use virtual_node::VirtualNode;
    /// let _div = VirtualNode::<()>::new_element("div");
    /// ```
    pub fn new_element<S>(tag: S) -> Self
    where
        S: Into<String>,
    {
        VirtualNode::Element(VirtualElement::new(tag))
    }

    /// Create a new virtual text node with the given text.
    ///
    /// These get patched into the DOM using `document.createTextNode`
    ///
    /// ```
    /// # use virtual_node::VirtualNode;
    /// let _text = VirtualNode::<()>::new_text("My text node");
    /// ```
    pub fn new_text<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        VirtualNode::Text(VirtualText::new(text.into()))
    }

    /// Return a [`VirtualElement`] reference, if this is an [`Element`] variant.
    ///
    /// [`VirtualElement`]: struct.VirtualElement.html
    /// [`Element`]: enum.VirtualNode.html#variant.Element
    pub fn as_elem(&self) -> Option<&VirtualElement<Dom>> {
        match self {
            VirtualNode::Element(ref element_node) => Some(element_node),
            _ => None,
        }
    }

    /// Return a mutable [`VirtualElement`] reference, if this is an [`Element`] variant.
    ///
    /// [`VirtualElement`]: struct.VirtualElement.html
    /// [`Element`]: enum.VirtualNode.html#variant.Element
    pub fn as_elem_mut(&mut self) -> Option<&mut VirtualElement<Dom>> {
        match self {
            VirtualNode::Element(ref mut element_node) => Some(element_node),
            _ => None,
        }
    }

    /// Return a [`VirtualText`] reference, if this is an [`Text`] variant.
    ///
    /// [`VirtualText`]: struct.VirtualText.html
    /// [`Text`]: enum.VirtualNode.html#variant.Text
    pub fn as_text(&self) -> Option<&VirtualText> {
        match self {
            VirtualNode::Text(ref text_node) => Some(text_node),
            _ => None,
        }
    }

    /// Return a mutable [`VText`] reference, if this is an [`Text`] variant.
    ///
    /// [`VText`]: struct.VText.html
    /// [`Text`]: enum.VirtualNode.html#variant.Text
    pub fn as_text_mut(&mut self) -> Option<&mut VirtualText> {
        match self {
            VirtualNode::Text(ref mut text_node) => Some(text_node),
            _ => None,
        }
    }

    /// Convert this `VirtualNode<DomA>` into a `VirtualNode<DomB>`.
    pub fn map_real_dom<New: RealDom>(
        self,
        // Used to be `impl Fn`, but switched to `&dyn Fn` after a user got an error:
        // ```
        // error: reached the recursion limit while instantiating `VirtualNode::<NewDomType>::map_real_dom::<Window, &&&&&&&&&&&&&&&&&&&...>
        // ```
        convert_event: &dyn Fn(EventHandler<Dom>) -> EventHandler<New>,
    ) -> VirtualNode<New> {
        match self {
            VirtualNode::Text(text) => VirtualNode::Text(text),
            VirtualNode::Element(elem) => {
                let children: Vec<VirtualNode<New>> = elem
                    .children
                    .into_iter()
                    .map(|old| old.map_real_dom::<New>(convert_event))
                    .collect();

                VirtualNode::Element(VirtualElement {
                    tag: elem.tag,
                    attrs: elem.attrs,
                    events: elem.events.convert_all(convert_event),
                    children,
                    special_attributes: elem.special_attributes,
                })
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

#[cfg(feature = "web")]
impl VirtualNode<web_sys::Window> {
    /// Create and return a [`web_sys::Node`] along with its events.
    pub fn create_dom_node(
        &self,
        events: &mut self::event::VirtualEvents<web_sys::Window>,
    ) -> (web_sys::Node, crate::event::VirtualEventNode) {
        match self {
            VirtualNode::Text(text_node) => (
                text_node.create_text_node().into(),
                events.create_text_node(),
            ),
            VirtualNode::Element(element_node) => {
                let (elem, events) = element_node.create_element_node(events);
                (elem.into(), events)
            }
        }
    }
}

// Blocked by `trait aliases` feature https://github.com/rust-lang/rust/issues/41517
// /// A [`View`] whose returned [`VirtualNode`]s can be rendered to a [`web_sys`] DOM.
// #[cfg(feature = "web")]
// pub trait ViewWebSys = View<web_sys::Window>;

/// A trait with common functionality for rendering front-end views.
pub trait View<Dom: RealDom> {
    /// Render a VirtualNode, or any IntoIter<VirtualNode>
    fn render(&self) -> VirtualNode<Dom>;
}

impl<V, Dom: RealDom> From<&V> for VirtualNode<Dom>
where
    V: View<Dom>,
{
    fn from(v: &V) -> Self {
        v.render()
    }
}

impl<Dom: RealDom> From<VirtualText> for VirtualNode<Dom> {
    fn from(other: VirtualText) -> Self {
        VirtualNode::Text(other)
    }
}

impl<Dom: RealDom> From<VirtualElement<Dom>> for VirtualNode<Dom> {
    fn from(other: VirtualElement<Dom>) -> Self {
        VirtualNode::Element(other)
    }
}

impl<Dom: RealDom> From<&str> for VirtualNode<Dom> {
    fn from(other: &str) -> Self {
        VirtualNode::new_text(other)
    }
}

impl<Dom: RealDom> From<String> for VirtualNode<Dom> {
    fn from(other: String) -> Self {
        VirtualNode::new_text(other.as_str())
    }
}

impl<Dom: RealDom> IntoIterator for VirtualNode<Dom> {
    type Item = VirtualNode<Dom>;
    // TODO: ::std::iter::Once<VirtualNode> to avoid allocation
    type IntoIter = ::std::vec::IntoIter<VirtualNode<Dom>>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self].into_iter()
    }
}

impl<Dom: RealDom> Into<::std::vec::IntoIter<VirtualNode<Dom>>> for VirtualNode<Dom> {
    fn into(self) -> ::std::vec::IntoIter<VirtualNode<Dom>> {
        self.into_iter()
    }
}

impl<Dom: RealDom> fmt::Debug for VirtualNode<Dom> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VirtualNode::Element(e) => write!(f, "Node::{:?}", e),
            VirtualNode::Text(t) => write!(f, "Node::{:?}", t),
        }
    }
}

// Turn a VirtualNode into an HTML string (delegate impl to variants)
impl<Dom: RealDom> fmt::Display for VirtualNode<Dom> {
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
        let node = VirtualNode::<()>::new_element("br");

        // No </br> since self closing tag
        assert_eq!(&node.to_string(), "<br>");
    }

    #[test]
    fn to_string() {
        let mut node = VirtualNode::Element(VirtualElement::<()>::new("div"));
        node.as_elem_mut()
            .unwrap()
            .attrs
            .insert("id".into(), "some-id".into());

        let mut child = VirtualNode::Element(VirtualElement::new("span"));

        let text = VirtualNode::Text(VirtualText::new("Hello world"));

        child.as_elem_mut().unwrap().children.push(text);

        node.as_elem_mut().unwrap().children.push(child);

        let expected = r#"<div id="some-id"><span>Hello world</span></div>"#;

        assert_eq!(node.to_string(), expected);
    }

    /// Verify that a boolean attribute is included in the string if true.
    #[test]
    fn boolean_attribute_true_shown() {
        let mut button = VirtualElement::<()>::new("button");
        button.attrs.insert("disabled".into(), true.into());

        let expected = "<button disabled></button>";
        let button = VirtualNode::Element(button).to_string();

        assert_eq!(button.to_string(), expected);
    }

    /// Verify that a boolean attribute is not included in the string if false.
    #[test]
    fn boolean_attribute_false_ignored() {
        let mut button = VirtualElement::<()>::new("button");
        button.attrs.insert("disabled".into(), false.into());

        let expected = "<button></button>";
        let button = VirtualNode::Element(button).to_string();

        assert_eq!(button.to_string(), expected);
    }
}
