//! A collection of functions that are useful for unit testing your html! views.

use crate::VirtualNode;

impl VirtualNode {
    /// Get a vector of all of the VirtualNode children / grandchildren / etc of
    /// your virtual_node that have a label that matches your filter.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # #[macro_use] extern crate virtual_dom_rs;  fn main() {
    ///
    /// let component = html! {<div>
    ///  <span label="hello",> {"Hi!"} </span>
    ///  <em label="world",> {"There!!"} </em>
    ///  <em label="hello",></em>
    /// </div> };
    ///
    /// let hello_nodes = component.filter_label(|label| {
    ///     label.contains("hello")
    /// });
    ///
    /// assert_eq!(hello_nodes.len(), 2);
    /// }
    /// ```
    pub fn filter_label<'a, F>(&'a self, filter: F) -> Vec<&'a VirtualNode>
    where
        F: Fn(&str) -> bool,
    {
        let mut descendants = vec![];

        for child in self.children.as_ref().unwrap() {
            get_descendants(&mut descendants, &child);
        }

        let mut filtered_descendants = vec![];
        for node in descendants.into_iter() {
            match node.props.get("label") {
                Some(label) => {
                    if filter(label) {
                        filtered_descendants.push(node);
                    }
                }
                None => {}
            };
        }

        filtered_descendants
    }

    /// Get a vector of all of the descendants of this VirtualNode
    /// that have the provided `filter`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # #[macro_use] extern crate virtual_dom_rs;  fn main() {
    ///
    /// let component = html! {<div>
    ///  <span label="hello",> {"Hi!"} </span>
    ///  <em label="world",> {"There!!"} </em>
    ///  <em label="hello",></em>
    /// </div> };
    ///
    /// let hello_nodes = component.filter_label_equals("hello");
    ///
    /// assert_eq!(hello_nodes.len(), 2);
    /// }
    /// ```
    pub fn filter_label_equals<'a>(&'a self, label: &str) -> Vec<&'a VirtualNode> {
        self.filter_label(|node_label| node_label == label)
    }
}

fn get_descendants<'a>(descendants: &mut Vec<&'a VirtualNode>, node: &'a VirtualNode) {
    descendants.push(node);

    for child in node.children.as_ref().unwrap() {
        get_descendants(descendants, child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // TODO: Move this test somewhere that we can use the `html!` macro
    //    #[test]
    //    fn filter_label() {
    //        let html = html! {
    //        // Should not pick up labels on the root node
    //        <div label="hello0",>
    //            // This node gets picked up
    //            <span label="hello1",>
    //            </span>
    //            // This node gets picked up
    //            <em label="hello2",>
    //                { "hello there :)!" }
    //            </em>
    //            <div label="world",></div>
    //        </div>
    //        };
    //
    //        let hello_nodes = html.filter_label(|label| label.contains("hello"));
    //
    //        assert_eq!(
    //            hello_nodes.len(),
    //            2,
    //            "2 elements with label containing 'hello'"
    //        );
    //    }

    #[test]
    fn label_equals() {
        let span = VirtualNode::new("span");

        let mut props = HashMap::new();
        props.insert("label".to_string(), "hello".to_string());
        let mut em = VirtualNode::new("em");
        em.props = props;

        let mut html = VirtualNode::new("div");
        html.children.as_mut().unwrap().push(span);
        html.children.as_mut().unwrap().push(em);

        let hello_nodes = html.filter_label_equals("hello");

        assert_eq!(hello_nodes.len(), 1);
    }
}
