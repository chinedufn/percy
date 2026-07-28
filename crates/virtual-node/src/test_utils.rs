//! A collection of functions that are useful for unit testing your html! views.

use crate::event::RealDom;
use crate::VirtualNode;

impl<Handle: RealDom> VirtualNode<Handle> {
    /// Get a vector of all of the VirtualNode children / grandchildren / etc of
    /// your virtual_node.
    ///
    /// Children are visited recursively depth first.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # #[macro_use] extern crate percy_dom;  fn main() {
    /// let component = html! {
    ///  <div>
    ///    <span> {"Hi!"} </span>
    ///    <em> {"There!!"} </em>
    ///    <div> {"My Friend"} </div>
    ///  </div>
    /// };
    ///
    /// let children = component.children_recursive();
    ///
    /// assert_eq!(children[2].tag(), "em");
    /// # }
    /// ```
    pub fn children_recursive<'a>(&'a self) -> Vec<&'a VirtualNode<Handle>> {
        let mut descendants: Vec<&'a VirtualNode<Handle>> = vec![];
        match self {
            VirtualNode::Text(_) => {}
            VirtualNode::Element(element_node) => {
                for child in element_node.children.iter() {
                    get_descendants(&mut descendants, child);
                }
            }
        }

        descendants.into_iter().collect()
    }
}

fn get_descendants<'a, Handle: RealDom>(
    descendants: &mut Vec<&'a VirtualNode<Handle>>,
    node: &'a VirtualNode<Handle>,
) {
    descendants.push(node);
    match node {
        VirtualNode::Text(_) => {}
        VirtualNode::Element(element_node) => {
            for child in element_node.children.iter() {
                get_descendants(descendants, child);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VirtualElement;

    /// Verify that we can return all of a node's descendants.
    #[test]
    fn children_recursive() {
        let span = VirtualNode::<()>::new_element("span");

        let mut em = VirtualElement::new("em");
        em.children.push(span);

        let mut html = VirtualElement::new("div");
        html.children.push(em.into());

        let html_node = VirtualNode::Element(html);

        assert_eq!(html_node.children_recursive().len(), 2);
    }
}
