use virtual_node::VirtualNode;
use webapis::*;
use Patch;

/// TODO: not implemented yet. This should return Vec<Patches> so that we can efficiently
///  patches the root node. Right now we just end up overwriting the root node.
pub fn diff<'a>(old_root: &VirtualNode, new_root: &mut VirtualNode) -> Vec<Patch<'a>> {
//    new_root.create_element()
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_nodes() {
        let old = html! {
        <div> <b></b> </div>
        };

        let mut new = html! {
        <div> <b></b> <new></new> </div>
        };

        let patches = diff(&old, &mut new);

        let new_node = new.children[1].borrow_mut();

        let expected_patch = Patch::AppendNodes(0, vec![&new_node]);

        assert_eq!(patches.len(), 1);
//        assert_eq!(patches[0], Patch::Ad)
    }
}
