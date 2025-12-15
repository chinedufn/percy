use js_sys::Reflect;
use std::cell::RefCell;
use std::collections::HashSet;
use std::collections::{HashMap, VecDeque};
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
/// that we desire. Also, update the `VirtualEvents` with the new virtual node's event callbacks.
///
/// This is usually used after diffing two virtual nodes.
// Tested in a browser in `percy-dom/tests`
pub fn patch<N: Into<Node>>(
    root_dom_node: N,
    new_vnode: &VirtualNode,
    virtual_events: &mut VirtualEvents,
    patches: &[Patch],
) -> Result<(), JsValue> {
    let root_events_node = virtual_events.root();

    let mut nodes_to_find = HashSet::new();

    for patch in patches {
        patch.insert_node_indices_to_find(&mut nodes_to_find);
    }

    let mut node_queue = VecDeque::new();
    node_queue.push_back(NodeToProcess {
        node: root_dom_node.into(),
        events_node: root_events_node.clone(),
        events_node_parent: None,
        node_idx: 0,
    });

    let mut ctx = PatchContext {
        next_node_idx: 1,
        nodes_to_find,
        found_nodes: HashMap::new(),
        events_id_for_old_node_idx: HashMap::new(),
        node_queue,
    };

    while ctx.nodes_to_find.len() >= 1 && ctx.node_queue.len() >= 1 {
        find_nodes(&mut ctx);
    }

    for patch in patches {
        let patch_node_idx = patch.old_node_idx();

        if let Some((_node, elem_or_text, events_elem)) = ctx.found_nodes.get(&patch_node_idx) {
            match elem_or_text {
                ElementOrText::Element(element) => {
                    apply_element_patch(&element, events_elem, &patch, virtual_events, &ctx)?
                }
                ElementOrText::Text(text_node) => {
                    apply_text_patch(&text_node, &patch, virtual_events, &events_elem.events_node)?;
                }
            };
        } else {
            // Right now this can happen if something outside of Percy goes into the DOM and
            //  deletes an element that is managed by Percy.
            panic!(
                "We didn't find the element or text node that we were supposed to patch ({}).",
                patch_node_idx
            )
        }
    }

    overwrite_events(new_vnode, root_events_node, virtual_events);

    Ok(())
}

struct PatchContext {
    next_node_idx: u32,
    nodes_to_find: HashSet<u32>,
    found_nodes: HashMap<u32, (Node, ElementOrText, EventsNodeAndParent)>,
    events_id_for_old_node_idx: HashMap<u32, ElementEventsId>,
    node_queue: VecDeque<NodeToProcess>,
}
struct NodeToProcess {
    node: Node,
    events_node: Rc<RefCell<VirtualEventNode>>,
    events_node_parent: Option<Rc<RefCell<VirtualEventNode>>>,
    node_idx: u32,
}
enum ElementOrText {
    Element(Element),
    Text(Text),
}
struct EventsNodeAndParent {
    events_node: Rc<RefCell<VirtualEventNode>>,
    parent: Option<Rc<RefCell<VirtualEventNode>>>,
}

impl PatchContext {
    fn store_found_node(&mut self, node_idx: u32, node: Node, events_node: EventsNodeAndParent) {
        self.nodes_to_find.remove(&node_idx);
        match node.node_type() {
            Node::ELEMENT_NODE => {
                let elem = ElementOrText::Element(node.clone().unchecked_into());
                self.found_nodes.insert(node_idx, (node, elem, events_node));
            }
            Node::TEXT_NODE => {
                let text = ElementOrText::Text(node.clone().unchecked_into());
                self.found_nodes.insert(node_idx, (node, text, events_node));
            }
            other => unimplemented!("Unsupported root node type: {}", other),
        }
    }
}

fn find_nodes(ctx: &mut PatchContext) {
    if ctx.nodes_to_find.len() == 0 {
        return;
    }

    let next = ctx.node_queue.pop_front();
    if next.is_none() {
        return;
    }

    let job = next.unwrap();
    let node = job.node;
    let events_node = job.events_node;
    let events_node_parent = job.events_node_parent;
    let cur_node_idx = job.node_idx;

    if let Some(events_elem) = events_node.borrow().as_element() {
        let events_id = events_elem.events_id();
        ctx.events_id_for_old_node_idx
            .insert(cur_node_idx, events_id);
    }

    if ctx.nodes_to_find.contains(&cur_node_idx) {
        let events = EventsNodeAndParent {
            events_node: events_node.clone(),
            parent: events_node_parent,
        };
        ctx.store_found_node(cur_node_idx, node.clone(), events);
    }

    // We use child_nodes() instead of children() because children() ignores text nodes
    let children = node.child_nodes();
    let child_node_count = children.length();

    if child_node_count == 0 {
        return;
    }

    let events_node_borrow = events_node.borrow();
    let events_node_elem = &events_node_borrow.as_element().unwrap();
    let mut next_child = events_node_elem.first_child();

    for i in 0..child_node_count {
        let child_node = children.item(i).unwrap();

        if !was_created_by_percy(&child_node) {
            continue;
        }

        let next_node_idx = ctx.next_node_idx;

        match child_node.node_type() {
            Node::ELEMENT_NODE | Node::TEXT_NODE => {
                let events_child_node = next_child.unwrap();
                next_child = events_child_node.borrow().next_sibling().cloned();

                ctx.node_queue.push_back(NodeToProcess {
                    node: child_node,
                    events_node: events_child_node,
                    events_node_parent: Some(events_node.clone()),
                    node_idx: next_node_idx,
                });

                ctx.next_node_idx += 1;
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

        let mut events_child = events_node.first_child();

        for child in elem.children.iter() {
            let e = events_child.unwrap();
            events_child = e.borrow().next_sibling().cloned();
            overwrite_events(child, e, virtual_events);
        }
    }
}

fn apply_element_patch(
    node: &Element,
    events_elem_and_parent: &EventsNodeAndParent,
    patch: &Patch,
    virtual_events: &mut VirtualEvents,
    ctx: &PatchContext,
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
                        // Use `set_checked` instead of `{set,remove}_attribute` for the `checked` attribute.
                        // The "checked" attribute only determines default checkedness,
                        // but `percy-dom` takes `checked` to specify the actual checkedness.
                        // See crates/percy-dom/tests/checked_attribute.rs for more info.
                        if *attrib_name == "checked" {
                            maybe_set_checked_property(node, *val_bool);
                        } else if *val_bool {
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
            new_node,
        } => {
            let (created_node, events) = new_node.create_dom_node(virtual_events);

            node.replace_with_with_node_1(&created_node)?;

            let mut events_elem = events_elem_and_parent.events_node.borrow_mut();
            events_elem.replace_with_node(events);

            Ok(())
        }
        Patch::InsertBefore {
            anchor_old_node_idx: _,
            new_nodes,
        } => {
            let parent = node.parent_element().unwrap();

            let events_parent = events_elem_and_parent.parent.as_ref().unwrap();

            for new_node in new_nodes {
                let (created_node, events) = new_node.create_dom_node(virtual_events);

                parent.insert_before(&created_node, Some(&node))?;
                events_parent.borrow_mut().insert_before(
                    Rc::new(RefCell::new(events)),
                    events_elem_and_parent.events_node.clone(),
                );
            }

            Ok(())
        }
        Patch::MoveNodesBefore {
            anchor_old_node_idx: _,
            to_move,
        } => {
            let parent = node.parent_element().unwrap();

            let events_parent = events_elem_and_parent.parent.as_ref().unwrap();
            let mut events_parent = events_parent.borrow_mut();

            for to_move_node in to_move {
                let (to_move_dom_node, _, to_move_node_events) =
                    ctx.found_nodes.get(to_move_node).unwrap();

                parent.insert_before(to_move_dom_node, Some(&node))?;

                events_parent.remove_node_from_siblings(&to_move_node_events.events_node);
                events_parent.insert_before(
                    to_move_node_events.events_node.clone(),
                    events_elem_and_parent.events_node.clone(),
                );
            }

            Ok(())
        }
        Patch::RemoveChildren {
            parent_old_node_idx: _,
            to_remove,
        } => {
            let parent = node;

            let events_elem = events_elem_and_parent.events_node.borrow_mut();
            let mut events_parent = events_elem;

            for idx in to_remove {
                let (node_to_remove, _, events_node_to_remove) = ctx.found_nodes.get(idx).unwrap();
                parent.remove_child(&node_to_remove)?;

                events_parent.remove_node_from_siblings(&events_node_to_remove.events_node);
            }

            Ok(())
        }
        Patch::AppendChildren {
            parent_old_node_idx: _,
            new_nodes,
        } => {
            let parent = &node;

            let events_elem = events_elem_and_parent.events_node.borrow_mut();
            let mut events_parent = events_elem;

            for new_node in new_nodes {
                let (created_node, events) = new_node.create_dom_node(virtual_events);

                parent.append_child(&created_node)?;

                events_parent
                    .as_element_mut()
                    .unwrap()
                    .append_child(Rc::new(RefCell::new(events)));
            }

            Ok(())
        }
        Patch::MoveToEndOfSiblings {
            parent_old_node_idx: _,
            siblings_to_move,
        } => {
            let parent = node;

            let events_elem = events_elem_and_parent.events_node.borrow_mut();
            let mut events_parent = events_elem;

            for node in siblings_to_move {
                let (dom_node_to_move, _, events_node_to_move) = ctx.found_nodes.get(node).unwrap();

                parent.append_child(&dom_node_to_move)?;

                events_parent.remove_node_from_siblings(&events_node_to_move.events_node);
                events_parent
                    .as_element_mut()
                    .unwrap()
                    .append_child(events_node_to_move.events_node.clone());
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
        Patch::CheckedAttributeUnchanged(_node_idx, value) => {
            maybe_set_checked_property(node, value.as_bool().unwrap());

            Ok(())
        }
        Patch::SpecialAttribute(special) => match special {
            PatchSpecialAttribute::CallOnCreateElemOnExistingNode(_node_idx, new_node) => {
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
            let events_id = ctx.events_id_for_old_node_idx.get(node_idx).unwrap();

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
            let events_id = ctx.events_id_for_old_node_idx.get(node_idx).unwrap();

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
            let events_id = ctx.events_id_for_old_node_idx.get(node_idx).unwrap();
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
            new_node,
        } => {
            let (elem, enode) = new_node.create_dom_node(events);
            node.replace_with_with_node_1(&elem)?;

            events_elem.borrow_mut().replace_with_node(enode);
        }
        other => {
            unreachable!(
                "Text nodes should only receive ChangeText or Replace patches, not {:?}.",
                other,
            )
        }
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

// See crates/percy-dom/tests/checked_attribute.rs
fn maybe_set_checked_property(node: &Element, checked: bool) {
    if let Some(input_node) = node.dyn_ref::<HtmlInputElement>() {
        input_node.set_checked(checked);
    }
}

// Looks for a property on the element. If it's there then this is a Percy element.
//
// TODO: We need to know not just if the node was created by Percy... but if it was created by
//  this percy-dom instance.. So give every PercyDom instance a random number and store that at the
//  virtual node marker property value.
fn was_created_by_percy(node: &Node) -> bool {
    let marker = Reflect::get(&node, &VIRTUAL_NODE_MARKER_PROPERTY.into()).unwrap();

    match marker.as_f64() {
        Some(_marker) => true,
        None => false,
    }
}
