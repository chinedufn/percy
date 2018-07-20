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
    AppendChildren(node_idx, Vec<&'a VirtualNode>),
    /// For a `node_i32`, remove all children besides the first `len`
    TruncateChildren(node_idx, usize),
    /// Replace a node with another node. This typically happens when a node's tag changes.
    /// ex: <div> becomes <span>
    Replace(node_idx, &'a VirtualNode),
    /// Add attributes that the new node has that the old node does not
    AddAttributes(node_idx, HashMap<&'a str, &'a str>),
    /// Remove attributes that the old node had that the new node doesn't
    RemoveAttributes(node_idx, Vec<&'a str>),
}

impl<'a> Patch<'a> {
    pub fn node_idx(&self) -> usize {
        match self {
            Patch::AppendChildren(node_idx, _) => *node_idx,
            Patch::TruncateChildren(node_idx, _) => *node_idx,
            Patch::Replace(node_idx, _) => *node_idx,
            Patch::AddAttributes(node_idx, _) => *node_idx,
            Patch::RemoveAttributes(node_idx, _) => *node_idx,
        }
    }
}

type node_idx = usize;

// TODO: Remove
macro_rules! clog {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

/// TODO: not implemented yet. This should use Vec<Patches> so that we can efficiently
///  patches the root node. Right now we just end up overwriting the root node.
pub fn patch(root_node: &Element, patches: &Vec<Patch>) {
    let mut cur_node_idx = 0;
    let mut cur_node = root_node;

    let mut child_node_count = cur_node.child_nodes().length() as usize;
//    cur_node.child_nodes().item(0).replace_with(&new_node.create_element());

    for patch in patches {
        let patch_node_idx = patch.node_idx();
        if cur_node_idx != patch_node_idx {
            let patch_node_idx_distance = patch_node_idx - cur_node_idx ;
            if patch_node_idx_distance < child_node_count {
                cur_node = cur_node.child_nodes().item(patch_node_idx_distance - 1);
            }
        }
        clog!("NODE INDEX {}", patch.node_idx());

        match patch {
            Patch::AddAttributes(node_idx, attributes) => {
                if *node_idx == cur_node_idx {
                    for (attrib_name, attrib_val) in attributes.iter() {
                        cur_node.set_attribute(attrib_name, attrib_val);
                    }
                }
            }
            Patch::Replace(node_idx, new_node) => {
                if *node_idx == cur_node_idx {
                    // TODO: We might already have a reference to the parent element from
                    // a previous iteration so in the future when we optimiz take advantage
                    // of that. After we have some benchmarks in place..
                    cur_node.replace_with(&new_node.create_element());
                }
            }
            _ => {}
        }
    }
}
