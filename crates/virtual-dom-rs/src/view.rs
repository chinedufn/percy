use crate::virtual_node::VirtualNode;

/// A trait with common functionality for rendering front-end views.
///
/// TODO: VirtualNode::from(impl View)
pub trait View {
    /// Render a VirtualNode
    fn render(&self) -> VirtualNode;
}
