use virtual_node::VirtualNode;
use webapis::*;
use Patch;

pub fn diff<'a>(old_root: &VirtualNode, new_root: &'a VirtualNode) -> Vec<Patch<'a>> {
    let mut patches = vec![];

    if old_root.tag != new_root.tag {
        patches.push(Patch::ReplaceNode(0, &new_root));
    }

    let old_root_child_count = old_root.children.as_ref().unwrap().len();
    let new_root_child_count = new_root.children.as_ref().unwrap().len();

    if old_root_child_count < new_root_child_count {
        let append_patch: Vec<&'a VirtualNode> = new_root.children.as_ref().unwrap()
            [old_root_child_count..]
            .iter()
            .collect();
        patches.push(Patch::AppendNodes(0, append_patch))
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
    fn replace_root_node() {
        test(DiffTestCase {
            old: html! { <div> </div> },
            new: html! { <span> </span> },
            expected: vec![Patch::ReplaceNode(0, &html! { <span></span> })],
            description: "Added a new node to the root node",
        });
    }

    #[test]
    fn add_nodes() {
        test(DiffTestCase {
            old: html! { <div> <b></b> </div> },
            new: html! { <div> <b></b> <new></new> </div> },
            expected: vec![Patch::AppendNodes(0, vec![&html! { <new></new> }])],
            description: "Added a new node to the root node",
        });
    }

    fn test(test_case: DiffTestCase) {
        let patches = diff(&test_case.old, &test_case.new);

        assert_eq!(patches, test_case.expected, "{}", test_case.description);
    }
}
