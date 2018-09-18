use percy_webapis::*;
use std::collections::HashMap;
use std::collections::HashSet;
use virtual_node::VirtualNode;

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
/// (0 being the root node, 1 being it's first child, 2 being it's first child's first child).
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
/// We'll revisit our indexing in the future when we optimize our diff/patch process.
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
    ChangeText(node_idx, &'a VirtualNode),
}

impl<'a> Patch<'a> {
    pub fn node_idx(&self) -> usize {
        match self {
            Patch::AppendChildren(node_idx, _) => *node_idx,
            Patch::TruncateChildren(node_idx, _) => *node_idx,
            Patch::Replace(node_idx, _) => *node_idx,
            Patch::AddAttributes(node_idx, _) => *node_idx,
            Patch::RemoveAttributes(node_idx, _) => *node_idx,
            Patch::ChangeText(node_idx, _) => *node_idx,
        }
    }
}

type node_idx = usize;

pub fn patch(root_node: &Element, patches: &Vec<Patch>) {
    let mut cur_node_idx = 0;
    let mut cur_node = root_node.clone();

    let mut nodes_to_find = HashSet::new();
    let mut nodes_to_patch = HashMap::new();

    for patch in patches {
        nodes_to_find.insert(patch.node_idx());
    }

    find_nodes(
        &root_node,
        &mut cur_node_idx,
        &mut nodes_to_find,
        &mut nodes_to_patch,
    );

    for patch in patches {
        let patch_node_idx = patch.node_idx();

        let node = nodes_to_patch.get(&patch_node_idx).unwrap();

        apply_patch(&node, &patch);
    }
}

fn find_nodes(
    root_node: &Element,
    cur_node_idx: &mut usize,
    nodes_to_find: &mut HashSet<usize>,
    nodes_to_patch: &mut HashMap<usize, Element>,
) {
    if nodes_to_find.len() == 0 {
        return;
    }

    if nodes_to_find.get(&cur_node_idx).is_some() {
        nodes_to_patch.insert(*cur_node_idx, root_node.clone());
        nodes_to_find.remove(&cur_node_idx);
    }

    *cur_node_idx += 1;

    let child_node_count = root_node.child_nodes().length() as usize;

    for i in 0..child_node_count {
        let node = root_node.child_nodes().item(i);
        find_nodes(&node, cur_node_idx, nodes_to_find, nodes_to_patch);
    }
}

fn apply_patch(node: &Element, patch: &Patch) {
    match patch {
        Patch::AddAttributes(_node_idx, attributes) => {
            for (attrib_name, attrib_val) in attributes.iter() {
                node.set_attribute(attrib_name, attrib_val);
            }
        }
        Patch::RemoveAttributes(_node_idx, attributes) => {
            for attrib_name in attributes.iter() {
                node.remove_attribute(attrib_name);
            }
        }
        Patch::Replace(_node_idx, new_node) => {
            node.replace_with(&new_node.create_element());
        }
        Patch::TruncateChildren(_node_idx, len) => {
            let count = node.child_nodes().length();
            for _ in *len as u32..count {
                node.remove_child(&node.last_child());
            }
        }
        Patch::ChangeText(_node_idx, new_node) => {
            node.set_node_value(&new_node.text.as_ref().unwrap());
        }
        Patch::AppendChildren(_node_idx, new_nodes) => {
            for new_node in new_nodes {
                node.append_child(&new_node.create_element());
            }
        }
    }
}
