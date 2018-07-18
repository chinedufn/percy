use virtual_node::VirtualNode;
use webapis::*;
use Patch;
use std::cmp::min;

static START_INDEX: usize = 0;

pub fn diff<'a>(old: &VirtualNode, new: &'a VirtualNode) -> Vec<Patch<'a>> {
    diff_recursive(&old, &new, &mut 0)
}

fn diff_recursive<'a, 'b>(old: &VirtualNode, new: &'a VirtualNode, cur_node_idx: &'b mut usize) -> Vec<Patch<'a>> {
    let mut patches = vec![];

    if old.tag != new.tag {
        patches.push(Patch::Replace(0, &new));
    }

    let old_children = old.children.as_ref().unwrap();
    let new_children = new.children.as_ref().unwrap();

    let old_child_count = old_children.len();
    let new_child_count = new_children.len();

    if new_child_count > old_child_count {
        let append_patch: Vec<&'a VirtualNode> = new_children
            [old_child_count..]
            .iter()
            .collect();
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
    fn add_nodes() {
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

    fn test(test_case: DiffTestCase) {
        let patches = diff(&test_case.old, &test_case.new);

        assert_eq!(patches, test_case.expected, "{}", test_case.description);
    }
}
