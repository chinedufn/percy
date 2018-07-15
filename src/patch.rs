use webapis::*;
use virtual_node::VirtualNode;

/// TODO: not implemented yet. This should use Vec<Patches> so that we can efficiently
///  patches the root node. Right now we just end up overwriting the root node.
pub fn patch (root_node: &Element, patches: &Element) {
    let parent = root_node.parent_element();
    parent.replace_child(patches, root_node);
}