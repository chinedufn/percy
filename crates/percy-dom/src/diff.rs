use crate::event::{EventHandler, EventName};
use crate::{AttributeValue, Patch, PatchSpecialAttribute};
use crate::{VElement, VirtualNode};
use std::cmp::min;
use std::collections::HashMap;
use std::mem;

/// Given two VirtualNode's generate Patch's that would turn the old virtual node's
/// real DOM node equivalent into the new VirtualNode's real DOM node equivalent.
pub fn diff<'a>(old: &'a VirtualNode, new: &'a VirtualNode) -> Vec<Patch<'a>> {
    diff_recursive(&old, &new, &mut 0, &mut 0)
}

fn diff_recursive<'a, 'b>(
    old: &'a VirtualNode,
    new: &'a VirtualNode,
    old_node_idx: &'b mut u32,
    new_node_idx: &'b mut u32,
) -> Vec<Patch<'a>> {
    let mut patches = vec![];

    let node_variants_different = mem::discriminant(old) != mem::discriminant(new);
    let mut element_tags_different = false;

    if let (VirtualNode::Element(old_element), VirtualNode::Element(new_element)) = (old, new) {
        element_tags_different = old_element.tag != new_element.tag;
    }

    let should_fully_replace_node = node_variants_different || element_tags_different;

    if should_fully_replace_node {
        if let Some(velem) = old.as_velement_ref() {
            if velem.events.has_events() {
                patches.push(Patch::RemoveAllManagedEventsWithNodeIdx(*old_node_idx));
            }
        }

        let replaced_old_idx = *old_node_idx;

        if let VirtualNode::Element(old_element_node) = old {
            for child in old_element_node.children.iter() {
                process_deleted_old_node_child(child, old_node_idx, &mut patches);
            }
        }

        patches.push(Patch::Replace {
            old_idx: replaced_old_idx,
            new_idx: *new_node_idx,
            new_node: new,
        });

        if let Some(velem) = old.as_velement_ref() {
            if velem.special_attributes.on_remove_element_key().is_some() {
                patches.push(Patch::SpecialAttribute(
                    PatchSpecialAttribute::CallOnRemoveElem(*old_node_idx, old),
                ));
            }
        }

        if let VirtualNode::Element(new_element_node) = new {
            for child in new_element_node.children.iter() {
                increment_idx_for_child(child, new_node_idx);
            }
        }

        return patches;
    }

    match (old, new) {
        (VirtualNode::Text(old_text), VirtualNode::Text(new_text)) => {
            if old_text != new_text {
                patches.push(Patch::ChangeText(*old_node_idx, &new_text));
            }
        }

        (VirtualNode::Element(old_element), VirtualNode::Element(new_element)) => {
            let mut attributes_to_add: HashMap<&str, &AttributeValue> = HashMap::new();
            let mut attributes_to_remove: Vec<&str> = vec![];

            let mut events_to_add = HashMap::new();
            let mut events_to_remove = vec![];

            find_attributes_to_add(
                *old_node_idx,
                &mut attributes_to_add,
                old_element,
                new_element,
                &mut patches,
            );

            find_attributes_to_remove(
                &mut attributes_to_add,
                &mut attributes_to_remove,
                old_element,
                new_element,
            );

            find_events_to_add(&mut events_to_add, old_element, new_element);
            find_events_to_remove(
                &mut events_to_add,
                &mut events_to_remove,
                old_element,
                new_element,
            );

            if attributes_to_add.len() > 0 {
                patches.push(Patch::AddAttributes(*old_node_idx, attributes_to_add));
            }
            if attributes_to_remove.len() > 0 {
                patches.push(Patch::RemoveAttributes(*old_node_idx, attributes_to_remove));
            }

            if events_to_remove.len() > 0 {
                patches.push(Patch::RemoveEvents(*old_node_idx, events_to_remove));
            }
            if events_to_add.len() > 0 {
                patches.push(Patch::AddEvents(*old_node_idx, events_to_add));
            }

            // FIXME: Move into function
            match (
                old_element.special_attributes.dangerous_inner_html.as_ref(),
                new_element.special_attributes.dangerous_inner_html.as_ref(),
            ) {
                (None, Some(_)) => {
                    patches.push(Patch::SpecialAttribute(
                        PatchSpecialAttribute::SetDangerousInnerHtml(*old_node_idx, new),
                    ));
                }
                (Some(old_inner), Some(new_inner)) => {
                    if old_inner != new_inner {
                        patches.push(Patch::SpecialAttribute(
                            PatchSpecialAttribute::SetDangerousInnerHtml(*old_node_idx, new),
                        ));
                    }
                }
                (Some(_), None) => {
                    patches.push(Patch::SpecialAttribute(
                        PatchSpecialAttribute::RemoveDangerousInnerHtml(*old_node_idx),
                    ));
                }
                (None, None) => {}
            };

            // FIXME: Move into function
            match (
                old_element.special_attributes.on_create_element_key(),
                new_element.special_attributes.on_create_element_key(),
            ) {
                (None, Some(_)) => {
                    patches.push(Patch::SpecialAttribute(
                        PatchSpecialAttribute::CallOnCreateElem(*old_node_idx, new),
                    ));
                }
                (Some(old_id), Some(new_id)) => {
                    if new_id != old_id {
                        patches.push(Patch::SpecialAttribute(
                            PatchSpecialAttribute::CallOnCreateElem(*old_node_idx, new),
                        ));
                    }
                }
                (Some(_), None) | (None, None) => {}
            };

            match (
                old_element.special_attributes.on_remove_element_key(),
                new_element.special_attributes.on_remove_element_key(),
            ) {
                (Some(_), None) => {
                    patches.push(Patch::SpecialAttribute(
                        PatchSpecialAttribute::CallOnRemoveElem(*old_node_idx, old),
                    ));
                }
                (Some(old_id), Some(new_id)) => {
                    if old_id != new_id {
                        patches.push(Patch::SpecialAttribute(
                            PatchSpecialAttribute::CallOnRemoveElem(*old_node_idx, old),
                        ));
                    }
                }
                _ => {}
            }

            let old_elem_has_events = old_element.events.has_events();
            let new_elem_has_events = new_element.events.has_events();

            if !old_elem_has_events && new_elem_has_events {
                patches.push(Patch::SetEventsId {
                    old_idx: *old_node_idx,
                    new_idx: *new_node_idx,
                });
            } else if old_elem_has_events && !new_elem_has_events {
                patches.push(Patch::RemoveEventsId(*old_node_idx));
            } else if old_elem_has_events && new_elem_has_events {
                if old_node_idx != new_node_idx {
                    patches.push(Patch::SetEventsId {
                        old_idx: *old_node_idx,
                        new_idx: *new_node_idx,
                    });
                }
            }

            generate_patches_for_children(
                old_node_idx,
                new_node_idx,
                old_element,
                new_element,
                &mut patches,
            );
        }
        (VirtualNode::Text(_), VirtualNode::Element(_))
        | (VirtualNode::Element(_), VirtualNode::Text(_)) => {
            unreachable!("Unequal variant discriminants should already have been handled");
        }
    };

    patches
}

/// Add attributes from the new element that are not already on the old one or that have changed.
fn find_attributes_to_add<'a>(
    cur_node_idx: u32,
    attributes_to_add: &mut HashMap<&'a str, &'a AttributeValue>,
    old_element: &VElement,
    new_element: &'a VElement,
    patches: &mut Vec<Patch<'a>>,
) {
    for (new_attr_name, new_attr_val) in new_element.attrs.iter() {
        match old_element.attrs.get(new_attr_name) {
            Some(ref old_attr_val) => {
                if old_attr_val != &new_attr_val {
                    attributes_to_add.insert(new_attr_name, new_attr_val);
                } else if new_attr_name == "value" {
                    patches.push(Patch::ValueAttributeUnchanged(cur_node_idx, new_attr_val));
                }
            }
            None => {
                attributes_to_add.insert(new_attr_name, new_attr_val);
            }
        };
    }
}

/// Remove attributes that were on the old element that are not present on the new element.
fn find_attributes_to_remove<'a>(
    attributes_to_add: &mut HashMap<&str, &AttributeValue>,
    attributes_to_remove: &mut Vec<&'a str>,
    old_element: &'a VElement,
    new_element: &VElement,
) {
    for (old_attr_name, old_attr_val) in old_element.attrs.iter() {
        if attributes_to_add.get(&old_attr_name[..]).is_some() {
            continue;
        };

        match new_element.attrs.get(old_attr_name) {
            Some(ref new_attr_val) => {
                if new_attr_val != &old_attr_val {
                    attributes_to_remove.push(old_attr_name);
                }
            }
            None => {
                attributes_to_remove.push(old_attr_name);
            }
        };
    }
}

/// Add attributes from the new element that are not already on the old one or that have changed.
fn find_events_to_add<'a>(
    events_to_add: &mut HashMap<&'a EventName, &'a EventHandler>,
    old_element: &VElement,
    new_element: &'a VElement,
) {
    for (new_event_name, new_event) in new_element.events.iter() {
        if !old_element.events.contains_key(new_event_name) {
            events_to_add.insert(new_event_name, new_event);
        }
    }
}

/// Remove non delegated that were on the old element that are not present on the new element.
fn find_events_to_remove<'a>(
    events_to_add: &mut HashMap<&'a EventName, &'a EventHandler>,
    events_to_remove: &mut Vec<(&'a EventName, &'a EventHandler)>,
    old_element: &'a VElement,
    new_element: &'a VElement,
) {
    for (old_event_name, old_event) in old_element.events.iter() {
        if events_to_add.contains_key(old_event_name) {
            continue;
        };
        if new_element.events.contains_key(old_event_name) {
            continue;
        }

        events_to_remove.push((old_event_name, old_event));
    }
}

fn generate_patches_for_children<'a, 'b>(
    old_node_idx: &'b mut u32,
    new_node_idx: &'b mut u32,
    old_element: &'a VElement,
    new_element: &'a VElement,
    patches: &mut Vec<Patch<'a>>,
) {
    let old_child_count = old_element.children.len();
    let new_child_count = new_element.children.len();

    let current_old_node_idx = *old_node_idx;

    if new_child_count < old_child_count {
        patches.push(Patch::TruncateChildren(
            current_old_node_idx,
            new_child_count,
        ));
    }

    let min_count = min(old_child_count, new_child_count);
    for index in 0..min_count {
        *old_node_idx += 1;
        *new_node_idx += 1;

        let old_child = &old_element.children[index];
        let new_child = &new_element.children[index];
        patches.append(&mut diff_recursive(
            &old_child,
            &new_child,
            old_node_idx,
            new_node_idx,
        ))
    }

    if new_child_count < old_child_count {
        for child in old_element.children[min_count..].iter() {
            process_deleted_old_node_child(child, old_node_idx, patches);
        }
    } else if new_child_count > old_child_count {
        let mut append_patch = vec![];

        for new_node in new_element.children[old_child_count..].iter() {
            *new_node_idx += 1;

            append_patch.push((*new_node_idx, new_node));

            if let Some(elem) = new_node.as_velement_ref() {
                for child in elem.children.iter() {
                    increment_idx_for_child(child, new_node_idx);
                }
            }
        }

        patches.push(Patch::AppendChildren {
            old_idx: current_old_node_idx,
            new_nodes: append_patch,
        })
    }
}

/// Increment the `cur_node_idx` to account for this deleted node.
///
/// Then iterate through all of its children, recursively, and increment the `cur_node_idx`.
///
/// Along the way we also push patches to remove all tracked events for deleted nodes
/// (if they had events).
fn process_deleted_old_node_child<'a>(
    old_node: &'a VirtualNode,
    cur_node_idx: &mut u32,
    patches: &mut Vec<Patch<'a>>,
) {
    *cur_node_idx += 1;
    if let VirtualNode::Element(element_node) = old_node {
        if element_node.events.len() > 0 {
            patches.push(Patch::RemoveAllManagedEventsWithNodeIdx(*cur_node_idx));
        }

        if element_node
            .special_attributes
            .on_remove_element_key()
            .is_some()
        {
            patches.push(Patch::SpecialAttribute(
                PatchSpecialAttribute::CallOnRemoveElem(*cur_node_idx, old_node),
            ));
        }

        for child in element_node.children.iter() {
            process_deleted_old_node_child(&child, cur_node_idx, patches);
        }
    }
}

/// Recursively increment the node idx for each child, depth first.
fn increment_idx_for_child(new_node: &VirtualNode, new_node_idx: &mut u32) {
    *new_node_idx += 1;

    if let VirtualNode::Element(element_node) = new_node {
        for child in element_node.children.iter() {
            increment_idx_for_child(child, new_node_idx);
        }
    }
}

#[cfg(test)]
mod diff_test_case;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventName;
    use crate::{html, EventAttribFn, PatchSpecialAttribute, VText, VirtualNode};
    use std::collections::HashMap;
    use std::rc::Rc;
    use virtual_node::IterableNodes;
    use wasm_bindgen::JsValue;

    use super::diff_test_case::*;

    #[test]
    fn replace_node() {
        DiffTestCase {
            old: html! { <div> </div> },
            new: html! { <span> </span> },
            expected: vec![Patch::Replace {
                old_idx: 0,
                new_idx: 0,
                new_node: &html! { <span></span> },
            }],
        }
        .test();
        DiffTestCase {
            old: html! { <div> <b></b> </div> },
            new: html! { <div> <strong></strong> </div> },
            expected: vec![Patch::Replace {
                old_idx: 1,
                new_idx: 1,
                new_node: &html! { <strong></strong> },
            }],
        }
        .test();
        DiffTestCase {
            old: html! { <div> <b>1</b> <b></b> </div> },
            new: html! { <div> <i>{"1"} {"2"}</i> <br /> </div>},
            expected: vec![
                Patch::Replace {
                    old_idx: 1,
                    new_idx: 1,
                    new_node: &html! { <i>{"1"} {"2"}</i> },
                },
                Patch::Replace {
                    old_idx: 3,
                    new_idx: 4,
                    new_node: &html! { <br /> },
                },
            ],
        }
        .test();
    }

    /// Verify that we use the proper new node idx when we replace a node.
    #[test]
    fn replace_node_proper_new_node_idx() {
        DiffTestCase {
            old: html! {
                <div>
                  <div><em></em></div>
                  <div></div>
                </div>
            },
            new: html! {
                <div>
                  <span></span>
                  <strong></strong>
                </div>
            },
            expected: vec![
                Patch::Replace {
                    old_idx: 1,
                    new_idx: 1,
                    new_node: &html! { <span></span> },
                },
                Patch::Replace {
                    old_idx: 3,
                    new_idx: 2,
                    new_node: &html! { <strong></strong> },
                },
            ],
        }
        .test();
    }

    #[test]
    fn add_children() {
        DiffTestCase {
            old: html! { <div> <b></b> </div> },
            new: html! { <div> <b></b> <span></span> </div> },
            expected: vec![Patch::AppendChildren {
                old_idx: 0,
                new_nodes: vec![(2, &html! { <span></span> })],
            }],
        }
        .test();
    }

    /// Verify that we use the proper new node idx for appended children.
    #[test]
    fn proper_new_node_idx_for_added_children() {
        DiffTestCase {
            old: html! {
                <div>
                  <span><em></em></span>
                  <div>
                    <br />
                  </div>
                </div>
            },
            new: html! {
                <div>
                  <i></i>
                  <div>
                    <br />
                    <div><br /></div>
                    <div></div>
                  </div>
                </div>
            },
            expected: vec![
                Patch::Replace {
                    old_idx: 1,
                    new_idx: 1,
                    new_node: &html! { <i></i>},
                },
                Patch::AppendChildren {
                    old_idx: 3,
                    new_nodes: vec![
                        (4, &html! { <div><br /></div> }),
                        (6, &html! { <div></div> }),
                    ],
                },
            ],
        }
        .test();
    }

    #[test]
    fn remove_nodes() {
        DiffTestCase {
            old: html! { <div> <b></b> <span></span> </div> },
            new: html! { <div> </div> },
            expected: vec![Patch::TruncateChildren(0, 0)],
        }
        .test();
        DiffTestCase {
            old: html! {
              <div>
                <span>
                  <b></b>
                  // This `i` tag will get removed
                  <i></i>
                </span>
                // This `strong` tag will get removed
                <strong></strong>
              </div>
            },
            new: html! {
              <div>
                <span>
                  <b></b>
                </span>
              </div>
            },
            expected: vec![Patch::TruncateChildren(0, 1), Patch::TruncateChildren(1, 1)],
        }
        .test();
        DiffTestCase {
            old: html! {
                <div>
                  <b>
                    <i></i>
                    <i></i>
                  </b>
                  <b></b>
                </div>
            },
            new: html! {
                <div>
                  <b>
                    <i></i>
                  </b>
                  <i></i>
                </div>
            },
            expected: vec![
                Patch::TruncateChildren(1, 1),
                Patch::Replace {
                    old_idx: 4,
                    new_idx: 3,
                    new_node: &html! { <i></i> },
                },
            ],
        }
        .test();
    }

    #[test]
    fn add_attributes() {
        let mut attributes = HashMap::new();
        let id = "hello".into();
        attributes.insert("id", &id);

        DiffTestCase {
            old: html! { <div> </div> },
            new: html! { <div id="hello"> </div> },
            expected: vec![Patch::AddAttributes(0, attributes.clone())],
        }
        .test();

        DiffTestCase {
            old: html! { <div id="foobar"> </div> },
            new: html! { <div id="hello"> </div> },
            expected: vec![Patch::AddAttributes(0, attributes)],
        }
        .test();
    }

    #[test]
    fn remove_attributes() {
        DiffTestCase {
            old: html! { <div id="hey-there"></div> },
            new: html! { <div> </div> },
            expected: vec![Patch::RemoveAttributes(0, vec!["id"])],
        }
        .test();
    }

    #[test]
    fn change_attribute() {
        let mut attributes = HashMap::new();
        let id = "changed".into();
        attributes.insert("id", &id);

        DiffTestCase {
            old: html! { <div id="hey-there"></div> },
            new: html! { <div id="changed"> </div> },
            expected: vec![Patch::AddAttributes(0, attributes)],
        }
        .test();
    }

    #[test]
    fn replace_text_node() {
        DiffTestCase {
            old: html! { Old },
            new: html! { New },
            expected: vec![Patch::ChangeText(0, &VText::new("New"))],
        }
        .test();
    }

    /// If an input or textarea has a value attribute we always push a patch for setting the value
    /// attribute so that we can replace anything that might have been typed into the field.
    #[test]
    fn always_pushes_patch_for_value() {
        DiffTestCase {
            old: html! { <input value="abc" /> },
            new: html! { <input value="abc" /> },
            expected: vec![Patch::ValueAttributeUnchanged(0, &"abc".into())],
        }
        .test();

        DiffTestCase {
            old: html! { <textarea value="abc" /> },
            new: html! { <textarea value="abc" /> },
            expected: vec![Patch::ValueAttributeUnchanged(0, &"abc".into())],
        }
        .test();

        DiffTestCase {
            old: html! { <textarea value="abc" /> },
            new: html! { <textarea value="def" /> },
            expected: vec![Patch::AddAttributes(
                0,
                vec![("value", &"def".into())].into_iter().collect(),
            )],
        }
        .test();
    }

    /// Verify that we push an on create elem patch if the new node has the special attribute
    /// and the old node does not.
    #[test]
    fn on_create_elem() {
        let old = VirtualNode::element("div");

        let mut new = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut new, "150");

        let mut expected = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut expected, "150");

        DiffTestCase {
            old,
            new,
            expected: vec![Patch::SpecialAttribute(
                PatchSpecialAttribute::CallOnCreateElem(0, &expected),
            )],
        }
        .test();
    }

    /// Verify that if two different nodes have the same on_create_elem unique identifiers we
    /// do not push a CallOnCreateElem patch.
    #[test]
    fn same_on_create_elem_id() {
        let mut old = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut old, "70");

        let mut new = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut new, "70");

        DiffTestCase {
            old,
            new,
            expected: vec![],
        }
        .test();
    }

    /// Verify that if two different nodes have different on_create_elem unique identifiers we push
    /// a patch to call the new on_create_elem.
    #[test]
    fn different_on_create_elem_id() {
        let mut old = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut old, "50");

        let mut new = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut new, "99");

        let mut expected = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut expected, "99");

        DiffTestCase {
            old,
            new,
            expected: vec![Patch::SpecialAttribute(
                PatchSpecialAttribute::CallOnCreateElem(0, &expected),
            )],
        }
        .test();
    }

    /// Verify that we push an on remove elem patch if the new node has the special attribute
    /// and the old node does not.
    #[test]
    fn on_remove_elem_for_replaced_elem() {
        let mut old = VirtualNode::element("div");
        set_on_remove_elem_with_unique_id(&mut old, "150");

        let expected = {
            let mut old = VirtualNode::element("div");
            set_on_remove_elem_with_unique_id(&mut old, "150");

            old
        };

        let new = VirtualNode::element("span");

        DiffTestCase {
            old,
            new,
            expected: vec![
                Patch::Replace {
                    old_idx: 0,
                    new_idx: 0,
                    new_node: &VirtualNode::element("span"),
                },
                Patch::SpecialAttribute(PatchSpecialAttribute::CallOnRemoveElem(0, &expected)),
            ],
        }
        .test();
    }

    /// Verify that we push on remove element patches for replaced children, their replaced
    /// children, etc.
    #[test]
    fn on_remove_elem_for_replaced_children_recursively() {
        let mut grandchild = VirtualNode::element("strong");
        set_on_remove_elem_with_unique_id(&mut grandchild, "key");

        let mut child = VirtualNode::element("em");
        set_on_remove_elem_with_unique_id(&mut child, "key");

        child.as_velement_mut().unwrap().children.push(grandchild);

        let old = html! {
            <div>
                {child}
            </div>
        };

        let expected_child = {
            let mut grandchild = VirtualNode::element("strong");
            set_on_remove_elem_with_unique_id(&mut grandchild, "key");

            let mut child = VirtualNode::element("em");
            set_on_remove_elem_with_unique_id(&mut child, "key");
            child.as_velement_mut().unwrap().children.push(grandchild);

            child
        };
        let expected_grandchild = {
            let mut grandchild = VirtualNode::element("strong");
            set_on_remove_elem_with_unique_id(&mut grandchild, "key");
            grandchild
        };

        let new = VirtualNode::element("span");

        DiffTestCase {
            old,
            new,
            expected: vec![
                Patch::SpecialAttribute(PatchSpecialAttribute::CallOnRemoveElem(
                    1,
                    &expected_child,
                )),
                Patch::SpecialAttribute(PatchSpecialAttribute::CallOnRemoveElem(
                    2,
                    &expected_grandchild,
                )),
                Patch::Replace {
                    old_idx: 0,
                    new_idx: 0,
                    new_node: &VirtualNode::element("span"),
                },
            ],
        }
        .test();
    }

    /// Verify that we push on remove element patches for truncated children, their children,
    /// etc.
    #[test]
    fn on_remove_elem_for_truncated_children_recursively() {
        let mut grandchild = VirtualNode::element("strong");
        set_on_remove_elem_with_unique_id(&mut grandchild, "key");

        let mut child = VirtualNode::element("em");
        set_on_remove_elem_with_unique_id(&mut child, "key");

        child.as_velement_mut().unwrap().children.push(grandchild);

        let old = html! {
            <div>
                <span></span>
                // Gets truncated.
                {child}
            </div>
        };

        let new = html! {
            <div>
                <span></span>
            </div>
        };

        let expected_child = {
            let mut grandchild = VirtualNode::element("strong");
            set_on_remove_elem_with_unique_id(&mut grandchild, "key");

            let mut child = VirtualNode::element("em");
            set_on_remove_elem_with_unique_id(&mut child, "key");
            child.as_velement_mut().unwrap().children.push(grandchild);

            child
        };
        let expected_grandchild = {
            let mut grandchild = VirtualNode::element("strong");
            set_on_remove_elem_with_unique_id(&mut grandchild, "key");
            grandchild
        };

        DiffTestCase {
            old,
            new,
            expected: vec![
                Patch::TruncateChildren(0, 1),
                Patch::SpecialAttribute(PatchSpecialAttribute::CallOnRemoveElem(
                    2,
                    &expected_child,
                )),
                Patch::SpecialAttribute(PatchSpecialAttribute::CallOnRemoveElem(
                    3,
                    &expected_grandchild,
                )),
            ],
        }
        .test();
    }

    /// Verify that when patching attributes, if the old has an on remove element callback but the
    /// new node does not, we call the on remove element callback.
    ///
    /// But only for that element, since the element's below it might not get removed from the
    /// DOM.
    #[test]
    fn new_node_does_not_have_on_remove_elem() {
        let old_child = on_remove_node_with_on_remove_child();
        let mut old = html! {
            <div>
                {old_child}
            </div>
        };
        set_on_remove_elem_with_unique_id(&mut old, "some-key");

        let expected = {
            let old_child = on_remove_node_with_on_remove_child();
            let mut old = html! {
                <div>
                    {old_child}
                </div>
            };
            set_on_remove_elem_with_unique_id(&mut old, "some-key");

            old
        };

        let new_child = on_remove_node_with_on_remove_child();
        let new = html! {
            <div>
                {new_child}
            </div>
        };

        DiffTestCase {
            old,
            new,
            expected: vec![Patch::SpecialAttribute(
                PatchSpecialAttribute::CallOnRemoveElem(0, &expected),
            )],
        }
        .test();
    }

    /// Verify that when patching attributes, if the old and new node are of the same tag type but
    /// have different on remove element ID, a patch is pushed.
    ///
    /// But only for that element, since the element's below it might not get removed from the
    /// DOM.
    #[test]
    fn different_on_remove_elem_id() {
        let old_child = on_remove_node_with_on_remove_child();
        let mut old = html! {
            <div>
                {old_child}
            </div>
        };
        set_on_remove_elem_with_unique_id(&mut old, "start");

        let expected = {
            let old_child = on_remove_node_with_on_remove_child();
            let mut old = html! {
                <div>
                    {old_child}
                </div>
            };
            set_on_remove_elem_with_unique_id(&mut old, "start");

            old
        };

        let new_child = on_remove_node_with_on_remove_child();
        let mut new = html! {
            <div>
                {new_child}
            </div>
        };
        set_on_remove_elem_with_unique_id(&mut new, "end");

        DiffTestCase {
            old,
            new,
            expected: vec![Patch::SpecialAttribute(
                PatchSpecialAttribute::CallOnRemoveElem(0, &expected),
            )],
        }
        .test();
    }

    /// Verify that if the old and new node have the same on remove element ID, no patch is pushed.
    #[test]
    fn same_on_remove_elem_id() {
        let mut old = VirtualNode::element("div");
        set_on_remove_elem_with_unique_id(&mut old, "same");

        let mut new = VirtualNode::element("div");
        set_on_remove_elem_with_unique_id(&mut new, "same");

        DiffTestCase {
            old,
            new,
            expected: vec![],
        }
        .test();
    }

    /// Verify that if the old node and new node have the same dangerous_inner_html we do not push
    /// an SetDangerousInnerHtml patch.
    #[test]
    fn same_dangerous_inner_html() {
        let mut old = VirtualNode::element("div");
        set_dangerous_inner_html(&mut old, "hi");

        let mut new = VirtualNode::element("div");
        set_dangerous_inner_html(&mut new, "hi");

        DiffTestCase {
            old,
            new,
            expected: vec![],
        }
        .test();
    }

    /// Verify that if the new node has dangerous_inner_html that is different from the old node's,
    /// we push a patch to set the new inner html.
    #[test]
    fn different_dangerous_inner_html() {
        let mut old = VirtualNode::element("div");
        set_dangerous_inner_html(&mut old, "old");

        let mut new = VirtualNode::element("div");
        set_dangerous_inner_html(&mut new, "new");

        let mut expected = VirtualNode::element("div");
        set_dangerous_inner_html(&mut expected, "new");

        DiffTestCase {
            old,
            new,
            expected: vec![Patch::SpecialAttribute(
                PatchSpecialAttribute::SetDangerousInnerHtml(0, &expected),
            )],
        }
        .test();
    }

    /// Verify that if the new node does not have dangerous_inner_html and the old node does, we
    /// push a patch to truncate all children along with a patch to push the new node's
    /// children.
    #[test]
    fn remove_dangerous_inner_html() {
        let mut old = VirtualNode::element("div");
        set_dangerous_inner_html(&mut old, "hi");

        let new = html! { <div><em></em></div> };

        DiffTestCase {
            old,
            new,
            expected: vec![
                Patch::SpecialAttribute(PatchSpecialAttribute::RemoveDangerousInnerHtml(0)),
                Patch::AppendChildren {
                    old_idx: 0,
                    new_nodes: vec![(1, &VirtualNode::element("em"))],
                },
            ],
        }
        .test();
    }

    /// Verify that if a node goes from no events to having at least one event, we create a patch
    /// to set the events ID on the dom node.
    #[test]
    fn set_events_id_if_events_added() {
        let old = VElement::new("div");

        let mut new = VElement::new("div");
        new.events.insert(onclick_name(), mock_event_handler());

        DiffTestCase {
            old: VirtualNode::Element(old),
            new: VirtualNode::Element(new),
            expected: vec![
                Patch::AddEvents(
                    0,
                    vec![(&EventName::ONCLICK, &mock_event_handler())]
                        .into_iter()
                        .collect(),
                ),
                Patch::SetEventsId {
                    old_idx: 0,
                    new_idx: 0,
                },
            ],
        }
        .test();
    }

    /// Verify that we set the proper old and new node indices in the set events ID patch.
    #[test]
    fn uses_correct_new_node_idx_in_set_events_id_patch() {
        let old = html! {
            <div>
                <em> <area /> </em>
                <div></div>
            </div>
        };

        let new = html! {
            <div>
                <span></span>
                <div onclick=||{}></div>
            </div>
        };

        DiffTestCase {
            old,
            new,
            expected: vec![
                Patch::Replace {
                    old_idx: 1,
                    new_idx: 1,
                    new_node: &VirtualNode::element("span"),
                },
                Patch::AddEvents(
                    3,
                    vec![(&EventName::ONCLICK, &mock_event_handler())]
                        .into_iter()
                        .collect(),
                ),
                Patch::SetEventsId {
                    old_idx: 3,
                    new_idx: 2,
                },
            ],
        }
        .test();
    }

    /// Verify that if a node already had a event and we are patching it with another
    /// event we do not create a patch for setting the events ID.
    #[test]
    fn does_not_set_events_id_if_already_had_events() {
        let mut old = VElement::new("div");
        old.events.insert(onclick_name(), mock_event_handler());

        let mut new = VElement::new("div");
        new.events.insert(onclick_name(), mock_event_handler());

        DiffTestCase {
            old: VirtualNode::Element(old),
            new: VirtualNode::Element(new),
            expected: vec![],
        }
        .test();
    }

    /// Verify that if an earlier node in the tree was replaced, and a later node has events, all
    /// nodes after it get their events ID increased based on the number of elements removed.
    #[test]
    fn resets_events_id_if_earlier_nodes_replaced() {
        let old = html! {
            <div>
                // This node gets replaced
                <span>
                    <em>
                        <area />
                    </em>
                </span>

                <strong onclick=|| {}>
                    <div></div>
                    <a onclick=|| {}></a>
                </strong>
            </div>
        };

        let new = html! {
            <div>
                <div></div>

                <strong onclick=|| {}>
                    <div></div>
                    <a onclick=|| {}></a>
                </strong>
            </div>
        };

        DiffTestCase {
            old,
            new,
            expected: vec![
                Patch::Replace {
                    old_idx: 1,
                    new_idx: 1,
                    new_node: &VirtualNode::element("div"),
                },
                Patch::SetEventsId {
                    old_idx: 4,
                    new_idx: 2,
                },
                Patch::SetEventsId {
                    old_idx: 6,
                    new_idx: 4,
                },
            ],
        }
        .test();
    }

    /// Verify that if 5 earlier node were replaced replaced by 5 different nodes, we do not
    /// reset the events ID for nodes that come after it since the total number of nodes has not
    /// changed.
    ///
    /// This test should also cover cases where the same number of earlier nodes are
    /// truncated / appended, since our implementation just checks whether or not the new node IDX
    /// is equal to the old node IDX.
    /// If not, then that node and every node after it needs its events ID reset
    /// (if they have events).
    #[test]
    fn does_not_reset_events_id_if_earlier_node_replaced_by_same_number_of_nodes() {
        let old = html! {
            <div>
                // This node gets replaced, but with the same number of nodes.
                <span>
                    <em>
                        <area />
                    </em>
                </span>

                <strong onclick=|| {}>
                    <div></div>
                    <a onclick=|| {}></a>
                </strong>
            </div>
        };

        let new = html! {
            <div>
                <div>
                    <ul>
                        <li> </li>
                    </ul>
                </div>

                <strong onclick=|| {}>
                    <div></div>
                    <a onclick=|| {}></a>
                </strong>
            </div>
        };

        DiffTestCase {
            old,
            new,
            expected: vec![Patch::Replace {
                old_idx: 1,
                new_idx: 1,
                new_node: &html! {<div> <ul> <li> </li> </ul> </div>},
            }],
        }
        .test();
    }

    /// Verify that if somewhere earlier in the tree there were child nodes truncated
    /// (so the net number of earlier nodes decreased) we push a patch to set the later node's
    /// events ID.
    #[test]
    fn resets_events_if_if_earlier_nodes_truncated() {
        let old = html! {
            <div>
                // This node gets its children truncated.
                <span>
                    <em></em>
                    <area />
                </span>

                <strong onclick=|| {}>
                    <div></div>
                    <a onclick=|| {}></a>
                </strong>
            </div>
        };

        let new = html! {
            <div>
                <span>
                    <em></em>
                </span>

                <strong onclick=|| {}>
                    <div></div>
                    <a onclick=|| {}></a>
                </strong>
            </div>
        };

        DiffTestCase {
            old,
            new,
            expected: vec![
                Patch::TruncateChildren(1, 1),
                Patch::SetEventsId {
                    old_idx: 4,
                    new_idx: 3,
                },
                Patch::SetEventsId {
                    old_idx: 6,
                    new_idx: 5,
                },
            ],
        }
        .test();
    }

    /// Verify that if somewhere earlier in the tree there were child nodes appended
    /// (so the net number of earlier nodes increased) we push a patch to set the later node's
    /// events ID.
    #[test]
    fn resets_events_if_if_earlier_nodes_appended() {
        let old = html! {
            <div>
                // This node gets its children appended to.
                <span>
                    <em></em>
                </span>

                <strong onclick=|| {}>
                    <div></div>
                    <a onclick=|| {}></a>
                </strong>
            </div>
        };

        let new = html! {
            <div>
                <span>
                    <em></em>
                    <area />
                </span>

                <strong onclick=|| {}>
                    <div></div>
                    <a onclick=|| {}></a>
                </strong>
            </div>
        };

        DiffTestCase {
            old,
            new,
            expected: vec![
                Patch::AppendChildren {
                    old_idx: 1,
                    new_nodes: vec![(3, &VirtualNode::element("area"))],
                },
                Patch::SetEventsId {
                    old_idx: 3,
                    new_idx: 4,
                },
                Patch::SetEventsId {
                    old_idx: 5,
                    new_idx: 6,
                },
            ],
        }
        .test();
    }

    /// Verify that if we previously had events but we no longer have any events we push a patch
    /// to remove the events ID.
    #[test]
    fn removes_events_id_if_no_more_events() {
        let mut old = VElement::new("div");
        old.events.insert(onclick_name(), mock_event_handler());

        let new = VElement::new("div");

        DiffTestCase {
            old: VirtualNode::Element(old),
            new: VirtualNode::Element(new),
            expected: vec![
                Patch::RemoveEvents(
                    0,
                    vec![(&EventName::ONCLICK, &mock_event_handler())]
                        .into_iter()
                        .collect(),
                ),
                Patch::RemoveEventsId(0),
            ],
        }
        .test();
    }

    /// Verify that if an element has added and removed multiple non-delegated events, the remove
    /// event listener patches come before the add event listener patches.
    /// This ensures that we can look up the old functions in the `EventsByNodeIdx` that we'll need
    /// to pass into .remove_event_listener() before the SetEventListeners patch overwrites those
    /// functions.
    #[test]
    fn remove_event_patches_come_before_add_event_patches() {
        let mut old = VElement::new("div");
        old.events.insert(oninput_name(), mock_event_handler());

        let mut new = VElement::new("div");
        new.events.insert(onmousemove_name(), mock_event_handler());

        DiffTestCase {
            old: VirtualNode::Element(old),
            new: VirtualNode::Element(new),
            expected: vec![
                Patch::RemoveEvents(0, vec![(&oninput_name(), &mock_event_handler())]),
                Patch::AddEvents(
                    0,
                    vec![(&onmousemove_name(), &mock_event_handler())]
                        .into_iter()
                        .collect(),
                ),
            ],
        }
        .test();
    }

    /// Verify that if a node has events but the node is replaced we push a patch to remove all
    /// of its events from the EventsByNodeIdx.
    /// We ensure that this event removal patch should come before the patch to replace the node,
    /// so that we don't accidentally remove events that were for the node that replaced it.
    #[test]
    fn remove_all_tracked_events_if_replaced() {
        let mut old = VElement::new("div");
        old.events.insert(oninput_name(), mock_event_handler());

        let new = VElement::new("some-other-element");

        DiffTestCase {
            old: VirtualNode::Element(old),
            new: VirtualNode::Element(new),
            expected: vec![
                Patch::RemoveAllManagedEventsWithNodeIdx(0),
                Patch::Replace {
                    old_idx: 0,
                    new_idx: 0,
                    new_node: &VirtualNode::Element(VElement::new("some-other-element")),
                },
            ],
        }
        .test();
    }

    /// Verify that if a node's ancestor (parent, grandparent, ..etc) was replaced we push a patch
    /// to remove all of its events from the EventsByNodeIdx.
    /// We ensure that this event removal patch should come before the patch to replace the node,
    /// so that we don't accidentally remove events that were for the node that replaced it.
    #[test]
    fn removes_tracked_events_if_ancestor_replaced() {
        // node idx 0
        let mut old = VElement::new("div");
        // node idx 1
        old.children.push(VirtualNode::Element(VElement::new("a")));
        // node idx 2
        old.children.push(VirtualNode::text("b"));

        // node idx 3
        let mut child_of_old = VElement::new("div");
        child_of_old
            .events
            .insert(oninput_name(), mock_event_handler());
        old.children.push(VirtualNode::Element(child_of_old));

        let new = VElement::new("some-other-element");

        DiffTestCase {
            old: VirtualNode::Element(old),
            new: VirtualNode::Element(new),
            expected: vec![
                Patch::RemoveAllManagedEventsWithNodeIdx(3),
                Patch::Replace {
                    old_idx: 0,
                    new_idx: 0,
                    new_node: &VirtualNode::Element(VElement::new("some-other-element")),
                },
            ],
        }
        .test();
    }

    /// Verify that if a child node is truncated and it had events we push a patch to remove all
    /// of its events from the EventsByNodeIdx
    #[test]
    fn remove_tracked_events_if_truncated() {
        let mut old = VElement::new("div");
        let mut child_of_old = VElement::new("div");
        child_of_old
            .events
            .insert(oninput_name(), mock_event_handler());
        old.children.push(VirtualNode::Element(child_of_old));

        let new = VElement::new("div");

        DiffTestCase {
            old: VirtualNode::Element(old),
            new: VirtualNode::Element(new),
            expected: vec![
                Patch::TruncateChildren(0, 0),
                Patch::RemoveAllManagedEventsWithNodeIdx(1),
            ],
        }
        .test();
    }

    fn set_on_create_elem_with_unique_id(node: &mut VirtualNode, on_create_elem_id: &'static str) {
        node.as_velement_mut()
            .unwrap()
            .special_attributes
            .set_on_create_element(on_create_elem_id, |_: web_sys::Element| {});
    }

    fn set_on_remove_elem_with_unique_id(node: &mut VirtualNode, on_remove_elem_id: &'static str) {
        node.as_velement_mut()
            .unwrap()
            .special_attributes
            .set_on_remove_element(on_remove_elem_id, |_: web_sys::Element| {});
    }

    fn set_dangerous_inner_html(node: &mut VirtualNode, html: &str) {
        node.as_velement_mut()
            .unwrap()
            .special_attributes
            .dangerous_inner_html = Some(html.to_string());
    }

    /// Return a node that has an on remove element function.
    ///
    /// This node has a child that also has an on remove element function.
    ///
    /// <div>
    ///   <div></div>
    /// </div>
    fn on_remove_node_with_on_remove_child() -> VirtualNode {
        let mut child = VirtualNode::element("div");
        set_on_remove_elem_with_unique_id(&mut child, "555");

        let mut node = VirtualNode::element("div");
        set_on_remove_elem_with_unique_id(&mut node, "666");

        node.as_velement_mut().unwrap().children.push(child);

        node
    }

    fn mock_event_handler() -> EventHandler {
        EventHandler::UnsupportedSignature(EventAttribFn(Rc::new(Box::new(JsValue::NULL))))
    }

    fn onclick_name() -> EventName {
        "onclick".into()
    }

    fn oninput_name() -> EventName {
        "oninput".into()
    }

    fn onmousemove_name() -> EventName {
        "onmousemove".into()
    }
}
