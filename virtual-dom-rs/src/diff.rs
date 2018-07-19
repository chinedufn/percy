use std::cmp::min;
use std::collections::HashMap;
use virtual_node::VirtualNode;
use webapis::*;
use Patch;

static START_INDEX: usize = 0;

pub fn diff<'a>(old: &'a VirtualNode, new: &'a VirtualNode) -> Vec<Patch<'a>> {
    diff_recursive(&old, &new, &mut 0)
}

fn diff_recursive<'a, 'b>(
    old: &'a VirtualNode,
    new: &'a VirtualNode,
    cur_node_idx: &'b mut usize,
) -> Vec<Patch<'a>> {
    let mut patches = vec![];

    if old.tag != new.tag {
        patches.push(Patch::Replace(0, &new));
        return patches;
    }

    let mut add_attributes: HashMap<&str, &str> = HashMap::new();
    let mut remove_attributes: Vec<&str> = vec![];

    // TODO: -> split out into func
    for (new_prop_name, new_prop_val) in new.props.iter() {
        match old.props.get(new_prop_name) {
            Some(ref old_prop_val) => {
                if old_prop_val != &new_prop_val {
                    add_attributes.insert(new_prop_name, new_prop_val);
                }
            }
            None => {
                add_attributes.insert(new_prop_name, new_prop_val);
            }
        };
    }

    // TODO: -> split out into func
    for (old_prop_name, old_prop_val) in old.props.iter() {
        if add_attributes.get(&old_prop_name[..]).is_some() {
            continue;
        };

        match new.props.get(old_prop_name) {
            Some(ref new_prop_val) => {
                if new_prop_val != &old_prop_val {
                    remove_attributes.push(old_prop_name);
                }
            }
            None => {
                remove_attributes.push(old_prop_name);
            }
        };
    }

    if add_attributes.len() > 0 {
        patches.push(Patch::AddAttributes(*cur_node_idx, add_attributes));
    }
    if remove_attributes.len() > 0 {
        patches.push(Patch::RemoveAttributes(*cur_node_idx, remove_attributes));
    }

    let old_children = old.children.as_ref().unwrap();
    let new_children = new.children.as_ref().unwrap();

    let old_child_count = old_children.len();
    let new_child_count = new_children.len();

    if new_child_count > old_child_count {
        let append_patch: Vec<&'a VirtualNode> = new_children[old_child_count..].iter().collect();
        patches.push(Patch::AppendChildren(*cur_node_idx, append_patch))
    }

    if new_child_count < old_child_count {
        patches.push(Patch::TruncateChildren(*cur_node_idx, new_child_count))
    }

    for index in 0..min(old_child_count, new_child_count) {
        *cur_node_idx = *cur_node_idx + 1;
        let old_child = &old_children[index];
        let new_child = &new_children[index];
        patches.append(&mut diff_recursive(&old_child, &new_child, cur_node_idx))
    }

    //    new_root.create_element()
    patches
}

#[cfg(test)]
mod tests {
    use super::*;
    use patch::BeforeAfterNthChild;
    use std::collections::HashMap;

    struct DiffTestCase<'a> {
        old: VirtualNode,
        new: VirtualNode,
        expected: Vec<Patch<'a>>,
        description: &'static str,
    }

    #[test]
    fn replace_node() {
        test(DiffTestCase {
            old: html! { <div> </div> },
            new: html! { <span> </span> },
            expected: vec![Patch::Replace(0, &html! { <span></span> })],
            description: "Replace the root if the tag changed",
        });
    }

    #[test]
    fn add_children() {
        test(DiffTestCase {
            old: html! { <div> <b></b> </div> },
            new: html! { <div> <b></b> <new></new> </div> },
            expected: vec![Patch::AppendChildren(0, vec![&html! { <new></new> }])],
            description: "Added a new node to the root node",
        });
    }

    #[test]
    fn remove_nodes() {
        test(DiffTestCase {
            old: html! { <div> <b></b> <span></span> </div> },
            new: html! { <div> </div> },
            expected: vec![Patch::TruncateChildren(0, 0)],
            description: "Remove all child nodes at and after child sibling index 1",
        });
        test(DiffTestCase {
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
            description: "Remove a child and a grandchild node",
        });
    }

    #[test]
    fn add_attributes() {
        let mut attributes = HashMap::new();
        attributes.insert("id", "hello");

        test(DiffTestCase {
            old: html! { <div> </div> },
            new: html! { <div id="hello", },
            expected: vec![Patch::AddAttributes(0, attributes.clone())],
            description: "Add attributes",
        });

        test(DiffTestCase {
            old: html! { <div id="foobar",> </div> },
            new: html! { <div id="hello", },
            expected: vec![Patch::AddAttributes(0, attributes)],
            description: "Change attribute",
        });
    }

    #[test]
    fn remove_attributes() {
        test(DiffTestCase {
            old: html! { <div id="hey-there",></div> },
            new: html! { <div> </div> },
            expected: vec![Patch::RemoveAttributes(0, vec!["id"])],
            description: "Add attributes",
        })
    }

    #[test]
    fn change_attribute() {
        let mut attributes = HashMap::new();
        attributes.insert("id", "changed");

        test(DiffTestCase {
            old: html! { <div id="hey-there",></div> },
            new: html! { <div id="changed",> </div> },
            expected: vec![Patch::AddAttributes(0, attributes)],
            description: "Add attributes",
        })
    }

    #[test]
    fn reorder_chldren() {
        let old_children = vec![
            html! { <div key="hello",></div> },
            html! { <div key="world",></div>},
        ];

        let new_children = vec![
            html! { <div key="world",></div> },
            html! { <div key="hello",></div>},
        ];

        test(DiffTestCase {
            old: html! { <div> { old_children } </div> },
            new: html! { <div> { new_children } </div> },
            expected: vec![Patch::RearrangeChildren(
                0,
                vec![BeforeAfterNthChild(0, 1), BeforeAfterNthChild(1, 0)],
            )],
            description: "Add attributes",
        })
    }

    fn test(test_case: DiffTestCase) {
        let patches = diff(&test_case.old, &test_case.new);

        assert_eq!(patches, test_case.expected, "{}", test_case.description);
    }
}
