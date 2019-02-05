use crate::patch::Patch;
use std::cmp::min;
use std::collections::HashMap;
use std::collections::HashSet;
use wasm_bindgen::JsCast;
use web_sys;
use web_sys::{Element, Node};

/// Apply all of the patches to our old root node in order to create the new root node
/// that we desire.
/// This is usually used after diffing two virtual nodes.
pub fn patch(root_node: Element, patches: &Vec<Patch>) {
    let mut cur_node_idx = 0;

    let mut nodes_to_find = HashSet::new();

    for patch in patches {
        nodes_to_find.insert(patch.node_idx());
    }

    let mut elements_to_patch = HashMap::new();
    let mut text_nodes_to_patch = HashMap::new();

    find_nodes(
        root_node,
        &mut cur_node_idx,
        &mut nodes_to_find,
        &mut elements_to_patch,
        &mut text_nodes_to_patch,
    );

    for patch in patches {
        let patch_node_idx = patch.node_idx();

        if let Some(element) = elements_to_patch.get(&patch_node_idx) {
            apply_element_patch(&element, &patch);
            continue;
        }

        if let Some(text_node) = text_nodes_to_patch.get(&patch_node_idx) {
            apply_text_patch(&text_node, &patch);
            continue;
        }

        unreachable!("Getting here means we didn't find the element or next node that we were supposed to patch.")
    }
}

fn find_nodes(
    root_node: Element,
    cur_node_idx: &mut usize,
    nodes_to_find: &mut HashSet<usize>,
    nodes_to_patch: &mut HashMap<usize, Element>,
    text_nodes_to_patch: &mut HashMap<usize, web_sys::Node>,
) {
    if nodes_to_find.len() == 0 {
        return;
    }

    // We use child_nodes() instead of children() because children() ignores text nodes
    let children = root_node.child_nodes();
    let child_node_count = children.length();

    if nodes_to_find.get(&cur_node_idx).is_some() {
        nodes_to_patch.insert(*cur_node_idx, root_node);
        nodes_to_find.remove(&cur_node_idx);
    }

    *cur_node_idx += 1;

    for i in 0..child_node_count {
        let node = children.item(i).unwrap();

        let element = node.dyn_into::<Element>();

        if element.is_ok() {
            find_nodes(
                element.ok().unwrap(),
                cur_node_idx,
                nodes_to_find,
                nodes_to_patch,
                text_nodes_to_patch,
            );
        } else {
            let text_or_comment_node = element.err().unwrap();

            // At this time we do not support user entered comment nodes, so if we see a comment
            // then it was a delimiter created by virtual-dom-rs in order to ensure that two
            // neighboring text nodes did not get merged into one by the browser. So we skip
            // over this virtual-dom-rs generated comment node.
            if text_or_comment_node.node_type() == Node::COMMENT_NODE {
                continue;
            }

            if nodes_to_find.get(&cur_node_idx).is_some() {
                let text_node = text_or_comment_node;
                text_nodes_to_patch.insert(*cur_node_idx, text_node);
            }

            *cur_node_idx += 1;
        }
    }
}

fn apply_element_patch(node: &Element, patch: &Patch) {
    let document = web_sys::window().unwrap().document().unwrap();

    match patch {
        Patch::AddAttributes(_node_idx, attributes) => {
            for (attrib_name, attrib_val) in attributes.iter() {
                node.set_attribute(attrib_name, attrib_val)
                    .expect("Set attribute on element");
            }
        }
        Patch::RemoveAttributes(_node_idx, attributes) => {
            for attrib_name in attributes.iter() {
                node.remove_attribute(attrib_name)
                    .expect("Remove attribute from element");
            }
        }
        Patch::Replace(_node_idx, new_node) => {
            if new_node.is_text_node() {
                node.replace_with_with_node_1(&new_node.create_text_node())
                    .expect("Replaced with text node");
            } else {
                node.replace_with_with_node_1(&new_node.create_element())
                    .expect("Replaced with element");
            }
        }
        Patch::TruncateChildren(_node_idx, num_children_remaining) => {
            let children = node.child_nodes();
            let mut child_count = children.length();

            // We skip over any separators that we placed between two text nodes
            //   -> `<!--ptns-->`
            //  and trim all children that come after our new desired `num_children_remaining`
            let mut non_separator_children_found = 0;

            for index in 0 as u32..child_count {
                let child = children
                    .get(min(index, child_count - 1))
                    .expect("Potential child to truncate");

                // If this is a comment node then we know that it is a `<!--ptns-->`
                // text node separator that was created in virtual_node/mod.rs.
                if child.node_type() == Node::COMMENT_NODE {
                    continue;
                }

                non_separator_children_found += 1;

                if non_separator_children_found <= *num_children_remaining as u32 {
                    continue;
                }

                node.remove_child(&child).expect("Truncated children");
                child_count -= 1;
            }
        }
        Patch::AppendChildren(_node_idx, new_nodes) => {
            let parent = &node;

            for new_node in new_nodes {
                if new_node.is_text_node() {
                    parent
                        .append_child(
                            &document
                                .create_text_node(
                                    new_node.text.as_ref().expect("Text node to append"),
                                )
                                .dyn_into::<web_sys::Node>()
                                .ok()
                                .expect("Appended text node"),
                        )
                        .expect("Append text node");
                } else {
                    parent
                        .append_child(&new_node.create_element())
                        .expect("Appended child element");
                }
            }
        }
        Patch::ChangeText(_node_idx, _new_node) => unreachable!(
            "Elements should not receive ChangeText patches. Those should go to Node's"
        ),
    }
}

fn apply_text_patch(node: &Node, patch: &Patch) {
    match patch {
        Patch::ChangeText(_node_idx, new_node) => {
            let text = new_node.text.as_ref().expect("New text to use");

            node.set_node_value(Some(text.as_str()));
        }
        _ => unreachable!(
            "Node's should only receive change text patches. All other patches go to Element's"
        ),
    }
}
