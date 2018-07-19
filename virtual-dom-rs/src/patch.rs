use std::collections::HashMap;
use virtual_node::VirtualNode;
use webapis::*;

/// A `Patch` encodes an operation that modifies a real DOM element.
///
/// To update the real DOM that a user sees you'll want to first diff your
/// old virtual dom and new virtual dom.
///
/// This diff operation will generate `Vec<Patch>` with zero or more patches that, when
/// applied to your real DOM, will make your real DOM look like your new virtual dom.
///
/// Each `Patch` has a u32 node index that helps us identify the real DOM node that it applies to.
///
/// Our old virtual dom's nodes are indexed depth first, as shown in this illustration
/// (0 being the root node, 1 being it's first child, 4 being it's second child).
///               .─.
///              ( 0 )
///               `┬'
///           ┌────┴──────┐
///           │           │
///           ▼           ▼
///          .─.         .─.
///         ( 1 )       ( 4 )
///          `┬'         `─'
///      ┌────┴───┐       │
///      │        │       ├─────┬─────┐
///      ▼        ▼       │     │     │
///     .─.      .─.      ▼     ▼     ▼
///    ( 2 )    ( 3 )    .─.   .─.   .─.
///     `─'      `─'    ( 5 ) ( 6 ) ( 7 )
///                      `─'   `─'   `─'
///
/// Indexing depth first allows us to say:
///
/// - Hmm.. Our patch operation applies to Node 5. Let's start from our root node 0 and look
/// at its children.
///
/// - node 0 has children 1 and 4. 5 is bigger than 4 so we can completely ignore node 1!
///
/// - Ok now let's look at node 4's children. Node 4 has a child Node 5. Perfect, let's patch it!
///
/// Had we used breadth first indexing in our example above
/// (parent 0, first child 1, second child 2) we'd need to traverse all of node 1's children
/// to see if Node 5 was there. Good thing we don't do that!
#[derive(Debug, PartialEq)]
pub enum Patch<'a> {
    /// Append a vector of child nodes to a parent node id.
    AppendChildren(node_id, Vec<&'a VirtualNode>),
    /// For a `node_i32`, remove all children besides the first `len`
    TruncateChildren(node_id, usize),
    /// Replace a node with another node. This typically happens when a node's tag changes.
    /// ex: <div> becomes <span>
    Replace(node_id, &'a VirtualNode),
    /// Add attributes that the new node has that the old node does not
    AddAttributes(node_id, HashMap<&'a str, &'a str>),
    /// Remove attributes that the old node had that the new node doesn't
    RemoveAttributes(node_id, Vec<&'a str>),
    /// If children have a `key="...",` property we'll compare the keys of the old and new children
    /// to see if any nodes were simply moved. This allows us to just re-append then in the
    /// correct place instead of needing to remove and then re-insert them.
    RearrangeChildren(node_id, Vec<BeforeAfterNthChild>), // This will allow us to support for  `key="..."` property for efficiently re-ordering lists
}
type node_id = usize;

/// Move a node's child from one location to another within it's siblings.
/// so BeforeAfterNthChild(0, 5) would move the 0th child to become the 5th child instead.
/// This is only supported if children have `key="...",` set
#[derive(Debug, PartialEq)]
pub struct BeforeAfterNthChild(pub usize, pub usize);

/// TODO: not implemented yet. This should use Vec<Patches> so that we can efficiently
///  patches the root node. Right now we just end up overwriting the root node.
pub fn patch(root_node: &Element, patches: &Element) {
    let parent = root_node.parent_element();
    parent.replace_child(patches, root_node);
}
