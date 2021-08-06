use crate::{AttributeValue, Patch, PatchSpecialAttribute};
use crate::{VElement, VirtualNode};
use std::cmp::min;
use std::collections::HashMap;
use std::mem;

/// Given two VirtualNode's generate Patch's that would turn the old virtual node's
/// real DOM node equivalent into the new VirtualNode's real DOM node equivalent.
pub fn diff<'a>(old: &'a VirtualNode, new: &'a VirtualNode) -> Vec<Patch<'a>> {
    diff_recursive(&old, &new, &mut 0)
}

fn diff_recursive<'a, 'b>(
    old: &'a VirtualNode,
    new: &'a VirtualNode,
    cur_node_idx: &'b mut usize,
) -> Vec<Patch<'a>> {
    let mut patches = vec![];
    let mut should_fully_replace_node = false;

    let node_variants_different = mem::discriminant(old) != mem::discriminant(new);
    if node_variants_different {
        should_fully_replace_node = true;
    }

    if let (VirtualNode::Element(old_element), VirtualNode::Element(new_element)) = (old, new) {
        let element_tags_different = old_element.tag != new_element.tag;
        if element_tags_different {
            should_fully_replace_node = true;
        }
    }

    if should_fully_replace_node {
        patches.push(Patch::Replace(*cur_node_idx, &new));
        if let VirtualNode::Element(old_element_node) = old {
            for child in old_element_node.children.iter() {
                increment_node_idx_for_children(child, cur_node_idx);
            }
        }
        return patches;
    }

    match (old, new) {
        (VirtualNode::Text(old_text), VirtualNode::Text(new_text)) => {
            if old_text != new_text {
                patches.push(Patch::ChangeText(*cur_node_idx, &new_text));
            }
        }

        (VirtualNode::Element(old_element), VirtualNode::Element(new_element)) => {
            let mut attributes_to_add: HashMap<&str, &AttributeValue> = HashMap::new();
            let mut attributes_to_remove: Vec<&str> = vec![];

            find_attributes_to_add(
                *cur_node_idx,
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

            if attributes_to_add.len() > 0 {
                patches.push(Patch::AddAttributes(*cur_node_idx, attributes_to_add));
            }
            if attributes_to_remove.len() > 0 {
                patches.push(Patch::RemoveAttributes(*cur_node_idx, attributes_to_remove));
            }

            // FIXME: Move into function
            match (
                old_element.special_attributes.dangerous_inner_html.as_ref(),
                new_element.special_attributes.dangerous_inner_html.as_ref(),
            ) {
                (None, Some(_)) => {
                    patches.push(Patch::SpecialAttribute(
                        PatchSpecialAttribute::SetDangerousInnerHtml(*cur_node_idx, new),
                    ));
                }
                (Some(old_inner), Some(new_inner)) => {
                    if old_inner != new_inner {
                        patches.push(Patch::SpecialAttribute(
                            PatchSpecialAttribute::SetDangerousInnerHtml(*cur_node_idx, new),
                        ));
                    }
                }
                (Some(_), None) => {
                    patches.push(Patch::SpecialAttribute(
                        PatchSpecialAttribute::RemoveDangerousInnerHtml(*cur_node_idx),
                    ));
                }
                (None, None) => {}
            };

            // FIXME: Move into function
            match (
                old_element.special_attributes.on_create_elem.as_ref(),
                new_element.special_attributes.on_create_elem.as_ref(),
            ) {
                (None, Some(_)) => {
                    patches.push(Patch::SpecialAttribute(
                        PatchSpecialAttribute::CallOnCreateElem(*cur_node_idx, new),
                    ));
                }
                (Some((old_id, _)), Some((new_id, _))) => {
                    if new_id != old_id {
                        patches.push(Patch::SpecialAttribute(
                            PatchSpecialAttribute::CallOnCreateElem(*cur_node_idx, new),
                        ));
                    }
                }
                (Some(_), None) | (None, None) => {}
            };

            generate_patches_for_children(cur_node_idx, old_element, new_element, &mut patches);
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
    cur_node_idx: usize,
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

fn generate_patches_for_children<'a>(
    cur_node_idx: &mut usize,
    old_element: &'a VElement,
    new_element: &'a VElement,
    patches: &mut Vec<Patch<'a>>,
) {
    let old_child_count = old_element.children.len();
    let new_child_count = new_element.children.len();

    if new_child_count > old_child_count {
        let append_patch: Vec<&'a VirtualNode> =
            new_element.children[old_child_count..].iter().collect();
        patches.push(Patch::AppendChildren(*cur_node_idx, append_patch))
    } else if new_child_count < old_child_count {
        patches.push(Patch::TruncateChildren(*cur_node_idx, new_child_count))
    }

    let min_count = min(old_child_count, new_child_count);
    for index in 0..min_count {
        *cur_node_idx = *cur_node_idx + 1;
        let old_child = &old_element.children[index];
        let new_child = &new_element.children[index];
        patches.append(&mut diff_recursive(&old_child, &new_child, cur_node_idx))
    }
    if new_child_count < old_child_count {
        for child in old_element.children[min_count..].iter() {
            increment_node_idx_for_children(child, cur_node_idx);
        }
    }
}

fn increment_node_idx_for_children(old: &VirtualNode, cur_node_idx: &mut usize) {
    *cur_node_idx += 1;
    if let VirtualNode::Element(element_node) = old {
        for child in element_node.children.iter() {
            increment_node_idx_for_children(&child, cur_node_idx);
        }
    }
}

#[cfg(test)]
mod diff_test_case;
#[cfg(test)]
use self::diff_test_case::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{html, wrap_closure, PatchSpecialAttribute, VText, VirtualNode};
    use std::collections::HashMap;

    #[test]
    fn replace_node() {
        DiffTestCase {
            old: html! { <div> </div> },
            new: html! { <span> </span> },
            expected: vec![Patch::Replace(0, &html! { <span></span> })],
        }
        .test();
        DiffTestCase {
            old: html! { <div> <b></b> </div> },
            new: html! { <div> <strong></strong> </div> },
            expected: vec![Patch::Replace(1, &html! { <strong></strong> })],
        }
        .test();
        DiffTestCase {
            old: html! { <div> <b>1</b> <b></b> </div> },
            new: html! { <div> <i>1</i> <i></i> </div>},
            expected: vec![
                Patch::Replace(1, &html! { <i>1</i> }),
                Patch::Replace(3, &html! { <i></i> }),
            ],
        }
        .test();
    }

    #[test]
    fn add_children() {
        DiffTestCase {
            old: html! { <div> <b></b> </div> },
            new: html! { <div> <b></b> <span></span> </div> },
            expected: vec![Patch::AppendChildren(0, vec![&html! { <span></span> }])],
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
            </div> },
            new: html! {
            <div>
             <span>
              <b></b>
             </span>
            </div> },
            expected: vec![Patch::TruncateChildren(0, 1), Patch::TruncateChildren(1, 1)],
        }
        .test();
        DiffTestCase {
            old: html! { <div> <b> <i></i> <i></i> </b> <b></b> </div> },
            new: html! { <div> <b> <i></i> </b> <i></i> </div>},
            expected: vec![
                Patch::TruncateChildren(1, 1),
                Patch::Replace(4, &html! { <i></i> }),
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
        set_on_create_elem_with_unique_id(&mut new, 150);

        DiffTestCase {
            old,
            new,
            expected: vec![Patch::SpecialAttribute(
                PatchSpecialAttribute::CallOnCreateElem(0, &VirtualNode::element("div")),
            )],
        }
        .test();
    }

    /// Verify that if two different nodes have the same on_create_elem unique identifiers we
    /// do not push a CallOnCreateElem patch.
    #[test]
    fn same_on_create_elem_id() {
        let mut old = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut old, 70);

        let mut new = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut new, 70);

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
        set_on_create_elem_with_unique_id(&mut old, 50);

        let mut new = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut new, 99);

        DiffTestCase {
            old,
            new,
            expected: vec![Patch::SpecialAttribute(
                PatchSpecialAttribute::CallOnCreateElem(0, &VirtualNode::element("div")),
            )],
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
                Patch::AppendChildren(0, vec![&VirtualNode::element("em")]),
            ],
        }
        .test();
    }

    fn set_on_create_elem_with_unique_id(node: &mut VirtualNode, on_create_elem_id: u32) {
        node.as_velement_mut()
            .unwrap()
            .special_attributes
            .on_create_elem = Some((on_create_elem_id, wrap_closure(|_: web_sys::Element| {})));
    }

    fn set_dangerous_inner_html(node: &mut VirtualNode, html: &str) {
        node.as_velement_mut()
            .unwrap()
            .special_attributes
            .dangerous_inner_html = Some(html.to_string());
    }
}
