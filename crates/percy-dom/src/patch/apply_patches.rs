use std::cmp::min;
use std::collections::HashMap;
use std::collections::HashSet;

use virtual_node::event::insert_non_delegated_event;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{Element, HtmlInputElement, HtmlTextAreaElement, Node, Text};

use crate::event::{EventsByNodeIdx, ManagedEvent, EVENTS_ID_PROP};
use crate::patch::Patch;
use crate::{AttributeValue, PatchSpecialAttribute, VirtualNode};

/// Apply all of the patches to our old root node in order to create the new root node
/// that we desire. Also, update the `EventsByNodeIdx` with the new virtual node's event callbacks.
///
/// This is usually used after diffing two virtual nodes.
pub fn patch<N: Into<Node>>(
    root_node: N,
    new_vnode: &VirtualNode,
    managed_events: &mut EventsByNodeIdx,
    patches: &[Patch],
) -> Result<(), JsValue> {
    let root_node: Node = root_node.into();

    let mut cur_node_idx = 0;

    let mut nodes_to_find = HashSet::new();

    for patch in patches {
        nodes_to_find.insert(patch.old_node_idx());
    }

    let mut element_nodes_to_patch = HashMap::new();
    let mut text_nodes_to_patch = HashMap::new();

    find_nodes(
        root_node,
        &mut cur_node_idx,
        &mut nodes_to_find,
        &mut element_nodes_to_patch,
        &mut text_nodes_to_patch,
    );

    for patch in patches {
        let patch_node_idx = patch.old_node_idx();

        if let Some(element) = element_nodes_to_patch.get(&patch_node_idx) {
            apply_element_patch(&element, &patch, managed_events)?;
            continue;
        }

        if let Some(text_node) = text_nodes_to_patch.get(&patch_node_idx) {
            apply_text_patch(&text_node, &patch, managed_events)?;
            continue;
        }

        unreachable!(
            "We didn't find the element or next node that we were supposed to patch ({}).",
            patch_node_idx
        )
    }

    overwrite_events(new_vnode, &mut 0, managed_events);

    Ok(())
}

fn find_nodes(
    root_node: Node,
    cur_node_idx: &mut u32,
    nodes_to_find: &mut HashSet<u32>,
    element_nodes_to_patch: &mut HashMap<u32, Element>,
    text_nodes_to_patch: &mut HashMap<u32, Text>,
) {
    if nodes_to_find.len() == 0 {
        return;
    }

    // We use child_nodes() instead of children() because children() ignores text nodes
    let children = root_node.child_nodes();
    let child_node_count = children.length();

    // If the root node matches, mark it for patching
    if nodes_to_find.get(&cur_node_idx).is_some() {
        match root_node.node_type() {
            Node::ELEMENT_NODE => {
                element_nodes_to_patch.insert(*cur_node_idx, root_node.unchecked_into());
            }
            Node::TEXT_NODE => {
                text_nodes_to_patch.insert(*cur_node_idx, root_node.unchecked_into());
            }
            other => unimplemented!("Unsupported root node type: {}", other),
        }
        nodes_to_find.remove(&cur_node_idx);
    }

    *cur_node_idx += 1;

    for i in 0..child_node_count {
        let node = children.item(i).unwrap();

        match node.node_type() {
            Node::ELEMENT_NODE => {
                find_nodes(
                    node,
                    cur_node_idx,
                    nodes_to_find,
                    element_nodes_to_patch,
                    text_nodes_to_patch,
                );
            }
            Node::TEXT_NODE => {
                if nodes_to_find.get(&cur_node_idx).is_some() {
                    text_nodes_to_patch.insert(*cur_node_idx, node.unchecked_into());
                }

                *cur_node_idx += 1;
            }
            Node::COMMENT_NODE => {
                // At this time we do not support user entered comment nodes, so if we see a comment
                // then it was a delimiter created by percy-dom in order to ensure that two
                // neighboring text nodes did not get merged into one by the browser. So we skip
                // over this percy-dom generated comment node.
            }
            _other => {
                // Ignoring unsupported child node type
                // TODO: What do we do with this situation?
            }
        }
    }
}

fn overwrite_events(node: &VirtualNode, node_idx: &mut u32, managed_events: &mut EventsByNodeIdx) {
    if let Some(elem) = node.as_velement_ref() {
        for (event_name, event) in elem.events.iter() {
            managed_events.overwrite_event_attrib_fn(*node_idx, event_name, event.clone());
        }

        for child in elem.children.iter() {
            *node_idx += 1;

            overwrite_events(child, node_idx, managed_events);
        }
    }
}

fn apply_element_patch(
    node: &Element,
    patch: &Patch,
    managed_events: &mut EventsByNodeIdx,
) -> Result<(), JsValue> {
    match patch {
        Patch::AddAttributes(_node_idx, attributes) => {
            for (attrib_name, attrib_val) in attributes.iter() {
                match attrib_val {
                    AttributeValue::String(val_str) => {
                        node.set_attribute(attrib_name, val_str)?;

                        if attrib_name == &"value" {
                            maybe_set_value_property(node, val_str)
                        }
                    }
                    AttributeValue::Bool(val_bool) => {
                        if *val_bool {
                            node.set_attribute(attrib_name, "")?;
                        } else {
                            node.remove_attribute(attrib_name)?;
                        }
                    }
                }
            }

            Ok(())
        }
        Patch::RemoveAttributes(_node_idx, attributes) => {
            for attrib_name in attributes.iter() {
                node.remove_attribute(attrib_name)?;
            }

            Ok(())
        }
        Patch::Replace {
            old_idx: _,
            new_idx,
            new_node,
        } => {
            let created_node = new_node.create_dom_node(*new_idx, managed_events);

            node.replace_with_with_node_1(&created_node)?;

            Ok(())
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

            Ok(())
        }
        Patch::AppendChildren {
            old_idx: _,
            new_nodes,
        } => {
            let parent = &node;

            for (new_idx, new_node) in new_nodes {
                let created_node = new_node.create_dom_node(*new_idx, managed_events);

                parent.append_child(&created_node)?;
            }

            Ok(())
        }
        Patch::ChangeText(_node_idx, _new_node) => {
            unreachable!("Elements should not receive ChangeText patches.")
        }
        Patch::ValueAttributeUnchanged(_node_idx, value) => {
            node.set_attribute("value", value.as_string().unwrap())?;
            maybe_set_value_property(node, value.as_string().unwrap());

            Ok(())
        }
        Patch::SpecialAttribute(special) => match special {
            PatchSpecialAttribute::CallOnCreateElem(_node_idx, new_node) => {
                new_node
                    .as_velement_ref()
                    .unwrap()
                    .special_attributes
                    .maybe_call_on_create_element(&node);

                Ok(())
            }
            PatchSpecialAttribute::CallOnRemoveElem(_, old_node) => {
                old_node
                    .as_velement_ref()
                    .unwrap()
                    .special_attributes
                    .maybe_call_on_remove_element(node);

                Ok(())
            }
            PatchSpecialAttribute::SetDangerousInnerHtml(_node_idx, new_node) => {
                let new_inner_html = new_node
                    .as_velement_ref()
                    .unwrap()
                    .special_attributes
                    .dangerous_inner_html
                    .as_ref()
                    .unwrap();

                node.set_inner_html(new_inner_html);

                Ok(())
            }
            PatchSpecialAttribute::RemoveDangerousInnerHtml(_node_idx) => {
                node.set_inner_html("");

                Ok(())
            }
        },
        Patch::RemoveEventsId(_) => {
            js_sys::Reflect::set(node, &EVENTS_ID_PROP.into(), &JsValue::UNDEFINED).unwrap();

            Ok(())
        }
        Patch::SetEventsId { old_idx, new_idx } => {
            js_sys::Reflect::set(
                node,
                &EVENTS_ID_PROP.into(),
                &format!("{}{}", managed_events.events_id_props_prefix(), new_idx).into(),
            )
            .unwrap();

            let move_over_old_events = old_idx != new_idx;

            if move_over_old_events {
                managed_events.move_events(old_idx, *new_idx);
            }

            Ok(())
        }
        Patch::AddEvents(node_idx, new_events) => {
            for (event_name, event) in new_events {
                if event_name.is_delegated() {
                    managed_events.insert_managed_event(
                        *node_idx,
                        (*event_name).clone(),
                        ManagedEvent::Delegated((*event).clone()),
                    );
                } else {
                    insert_non_delegated_event(node, event_name, event, *node_idx, managed_events);
                }
            }

            Ok(())
        }
        Patch::RemoveEvents(node_idx, events) => {
            for (event_name, _event) in events {
                if !event_name.is_delegated() {
                    let managed = managed_events.remove_managed_event(node_idx, event_name);
                    match managed {
                        ManagedEvent::NonDelegated(_, wrapper) => {
                            node.remove_event_listener_with_callback(
                                event_name.without_on_prefix(),
                                wrapper.as_ref().as_ref().unchecked_ref(),
                            )
                            .unwrap();
                        }
                        _ => unreachable!(),
                    };
                }

                managed_events.remove_event_handler(node_idx, event_name);
            }

            Ok(())
        }
        Patch::RemoveAllManagedEventsWithNodeIdx(node_idx) => {
            managed_events.remove_node(node_idx);
            Ok(())
        }
    }
}

fn apply_text_patch(
    node: &Text,
    patch: &Patch,
    events: &mut EventsByNodeIdx,
) -> Result<(), JsValue> {
    match patch {
        Patch::ChangeText(_node_idx, new_node) => {
            node.set_node_value(Some(&new_node.text));
        }
        Patch::Replace {
            old_idx: _,
            new_idx,
            new_node,
        } => {
            node.replace_with_with_node_1(&new_node.create_dom_node(*new_idx, events))?;
        }
        other => unreachable!(
            "Text nodes should only receive ChangeText or Replace patches, not {:?}.",
            other,
        ),
    };

    Ok(())
}

// See crates/percy-dom/tests/value_attribute.rs
fn maybe_set_value_property(node: &Element, value: &str) {
    if let Some(input_node) = node.dyn_ref::<HtmlInputElement>() {
        input_node.set_value(value);
    } else if let Some(textarea_node) = node.dyn_ref::<HtmlTextAreaElement>() {
        textarea_node.set_value(value)
    }
}
