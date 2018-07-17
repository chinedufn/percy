use virtual_node::VirtualNode;
use webapis::*;
use Patch;

pub fn diff<'a>(old_root: &VirtualNode, new_root: &'a mut VirtualNode) -> Vec<Patch<'a>> {
    let mut patches = vec![];

    let old_root_child_count = old_root.children.as_ref().unwrap().len();
    let new_root_child_count = new_root.children.as_ref().unwrap().len();

    if old_root_child_count < new_root_child_count {
        let append_patch: Vec<&'a VirtualNode> = new_root.children.as_ref().unwrap()[old_root_child_count..]
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

    #[test]
    fn replace_root_node() {
        let old = html! {
        <div> </div>
        };

        let mut new = html! {
        <span> </span>
        };

        panic!();
    }

    #[test]
    fn add_nodes() {
        let old = html! {
        <div> <b></b> </div>
        };

        let mut new = html! {
        <div> <b></b> <new></new> </div>
        };

        let patches = diff(&old, &mut new);

//        let new_node = &new.children[1];
//
//        let expected_patch = Patch::AppendNodes(0, vec![&new_node]);
//
//        assert_eq!(patches.len(), 1);
        //        assert_eq!(patches[0], Patch::Ad)
    }
}
