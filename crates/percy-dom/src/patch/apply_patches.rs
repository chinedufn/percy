use js_sys::Reflect;
use std::cell::RefCell;
use std::cmp::min;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use virtual_node::event::{insert_non_delegated_event, ElementEventsId, VirtualEventNode};
use virtual_node::VIRTUAL_NODE_MARKER_PROPERTY;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{Element, HtmlInputElement, HtmlTextAreaElement, Node, Text};

use crate::event::VirtualEvents;
use crate::patch::Patch;
use crate::{AttributeValue, PatchSpecialAttribute, VirtualNode};

/// Apply all of the patches to our old root node in order to create the new root node
/// that we desire. Also, update the `EventsByNodeIdx` with the new virtual node's event callbacks.
///
/// This is usually used after diffing two virtual nodes.
// Tested in a browser in `percy-dom/tests`
pub fn patch<N: Into<Node>>(
    root_node: N,
    new_vnode: &VirtualNode,
    virtual_events: &mut VirtualEvents,
    patches: &[Patch],
) -> Result<(), JsValue> {
    let root_node: Node = root_node.into();

    let root_events_node = virtual_events.root();

    let mut cur_node_idx = 0;

    let mut nodes_to_find = HashSet::new();

    for patch in patches {
        nodes_to_find.insert(patch.old_node_idx());
    }

    let mut element_nodes_to_patch = HashMap::new();
    let mut text_nodes_to_patch = HashMap::new();

    let mut events_id_for_old_node_idx: HashMap<u32, ElementEventsId> = HashMap::new();

    find_nodes(
        root_node,
        root_events_node.clone(),
        &mut cur_node_idx,
        &mut nodes_to_find,
        &mut element_nodes_to_patch,
        &mut text_nodes_to_patch,
        &mut events_id_for_old_node_idx,
    );

    for patch in patches {
        let patch_node_idx = patch.old_node_idx();

        if let Some((element, events_elem)) = element_nodes_to_patch.get(&patch_node_idx) {
            apply_element_patch(
                &element,
                &mut events_elem.borrow_mut(),
                &patch,
                virtual_events,
                &events_id_for_old_node_idx,
            )?;
            continue;
        }

        if let Some((text_node, events_elem)) = text_nodes_to_patch.get(&patch_node_idx) {
            apply_text_patch(&text_node, &patch, virtual_events, events_elem)?;
            continue;
        }

        unreachable!(
            "We didn't find the element or next node that we were supposed to patch ({}).",
            patch_node_idx
        )
    }

    overwrite_events(new_vnode, root_events_node, virtual_events);

    Ok(())
}

fn find_nodes(
    current_node: Node,
    events_node: Rc<RefCell<VirtualEventNode>>,
    cur_node_idx: &mut u32,
    nodes_to_find: &mut HashSet<u32>,
    element_nodes_to_patch: &mut HashMap<u32, (Element, Rc<RefCell<VirtualEventNode>>)>,
    text_nodes_to_patch: &mut HashMap<u32, (Text, Rc<RefCell<VirtualEventNode>>)>,
    events_id_for_old_node_idx: &mut HashMap<u32, ElementEventsId>,
) {
    if nodes_to_find.len() == 0 {
        return;
    }

    // We use child_nodes() instead of children() because children() ignores text nodes
    let children = current_node.child_nodes();
    let child_node_count = children.length();

    if let Some(events_elem) = events_node.borrow().as_element() {
        let events_id = events_elem.events_id();
        events_id_for_old_node_idx.insert(*cur_node_idx, events_id);
    }

    // If the root node matches, mark it for patching
    if nodes_to_find.contains(&cur_node_idx) {
        match current_node.node_type() {
            Node::ELEMENT_NODE => {
                element_nodes_to_patch.insert(
                    *cur_node_idx,
                    (current_node.unchecked_into(), events_node.clone()),
                );
            }
            Node::TEXT_NODE => {
                text_nodes_to_patch.insert(
                    *cur_node_idx,
                    (current_node.unchecked_into(), events_node.clone()),
                );
            }
            other => unimplemented!("Unsupported root node type: {}", other),
        }
        nodes_to_find.remove(&cur_node_idx);
    }

    *cur_node_idx += 1;

    if child_node_count == 0 {
        return;
    }

    let events_node = events_node.borrow();
    let events_node_children = &events_node.as_element().unwrap().children();

    let mut child_idx = 0;
    for i in 0..child_node_count {
        let child_node = children.item(i).unwrap();
        if !was_created_by_percy(&child_node) {
            continue;
        }

        let events_child_node = events_node_children[child_idx].clone();

        match child_node.node_type() {
            Node::ELEMENT_NODE => {
                find_nodes(
                    child_node,
                    events_child_node,
                    cur_node_idx,
                    nodes_to_find,
                    element_nodes_to_patch,
                    text_nodes_to_patch,
                    events_id_for_old_node_idx,
                );
                child_idx += 1;
            }
            Node::TEXT_NODE => {
                if nodes_to_find.get(&cur_node_idx).is_some() {
                    text_nodes_to_patch.insert(
                        *cur_node_idx,
                        (child_node.unchecked_into(), events_child_node),
                    );
                }

                *cur_node_idx += 1;
                child_idx += 1;
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
        };
    }
}

fn overwrite_events(
    node: &VirtualNode,
    events_node: Rc<RefCell<VirtualEventNode>>,
    virtual_events: &mut VirtualEvents,
) {
    if let Some(elem) = node.as_velement_ref() {
        let events_node = events_node.borrow();
        let events_node = events_node.as_element().unwrap();
        let events_id = events_node.events_id();

        for (event_name, event) in elem.events.iter() {
            virtual_events.overwrite_event_attrib_fn(&events_id, event_name, event.clone());
        }

        for (child_idx, child) in elem.children.iter().enumerate() {
            let events_child = events_node.children()[child_idx].clone();

            overwrite_events(child, events_child, virtual_events);
        }
    }
}

fn apply_element_patch(
    node: &Element,
    events_elem: &mut VirtualEventNode,
    patch: &Patch,
    virtual_events: &mut VirtualEvents,
    events_id_for_old_node_idx: &HashMap<u32, ElementEventsId>,
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
            new_idx: _,
            new_node,
        } => {
            let (created_node, events) = new_node.create_dom_node(virtual_events);

            node.replace_with_with_node_1(&created_node)?;
            *events_elem = events;

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

            events_elem
                .as_element_mut()
                .unwrap()
                .truncate_children(*num_children_remaining);

            Ok(())
        }
        Patch::AppendChildren {
            old_idx: _,
            new_nodes,
        } => {
            let parent = &node;

            for (_node_idx, new_node) in new_nodes {
                let (created_node, events) = new_node.create_dom_node(virtual_events);

                parent.append_child(&created_node)?;
                events_elem.as_element_mut().unwrap().push_child(events);
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
        Patch::AddEvents(node_idx, new_events) => {
            let events_id = events_id_for_old_node_idx.get(node_idx).unwrap();

            for (event_name, event) in new_events {
                if event_name.is_delegated() {
                    virtual_events.insert_event(
                        *events_id,
                        (*event_name).clone(),
                        (*event).clone(),
                        None,
                    );
                } else {
                    insert_non_delegated_event(node, event_name, event, *events_id, virtual_events);
                }
            }

            Ok(())
        }
        Patch::RemoveEvents(node_idx, events) => {
            let events_id = events_id_for_old_node_idx.get(node_idx).unwrap();

            for (event_name, _event) in events {
                if !event_name.is_delegated() {
                    let wrapper =
                        virtual_events.remove_non_delegated_event_wrapper(events_id, event_name);
                    node.remove_event_listener_with_callback(
                        event_name.without_on_prefix(),
                        wrapper.as_ref().as_ref().unchecked_ref(),
                    )
                    .unwrap();
                }

                virtual_events.remove_event_handler(events_id, event_name);
            }

            Ok(())
        }
        Patch::RemoveAllVirtualEventsWithNodeIdx(node_idx) => {
            let events_id = events_id_for_old_node_idx.get(node_idx).unwrap();
            virtual_events.remove_node(events_id);
            Ok(())
        }
    }
}

fn apply_text_patch(
    node: &Text,
    patch: &Patch,
    events: &mut VirtualEvents,
    events_elem: &Rc<RefCell<VirtualEventNode>>,
) -> Result<(), JsValue> {
    match patch {
        Patch::ChangeText(_node_idx, new_node) => {
            node.set_node_value(Some(&new_node.text));
        }
        Patch::Replace {
            old_idx: _,
            new_idx: _,
            new_node,
        } => {
            let (elem, enode) = new_node.create_dom_node(events);
            node.replace_with_with_node_1(&elem)?;

            *events_elem.borrow_mut() = enode;
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

// Looks for a property on the element. If it's there then this is a Percy element.
//
// TODO: We need to know not just if the node was created by Percy... but if it was created by
//  this percy-dom instance.. So give every PercyDom instance a random number and store that at the
//  virtual node marker property value.
fn was_created_by_percy(node: &web_sys::Node) -> bool {
    let marker = Reflect::get(&node, &VIRTUAL_NODE_MARKER_PROPERTY.into()).unwrap();

    match marker.as_f64() {
        Some(_marker) => true,
        None => false,
    }
}
