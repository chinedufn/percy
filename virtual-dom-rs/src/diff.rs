use virtual_node::VirtualNode;
use webapis::*;
use Patch;

/// TODO: not implemented yet. This should return Vec<Patches> so that we can efficiently
///  patches the root node. Right now we just end up overwriting the root node.
pub fn diff(old_root: &VirtualNode, new_root: &mut VirtualNode) -> Element {
    new_root.create_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_nodes() {
        let old = html! {
        <div> <b></b> </div>
        };

        let new = html! {
        <div> <b></b> </div>
        };
    }
}
