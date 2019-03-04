use crate::VirtualNode;

/// A trait with common functionality for rendering front-end views.
pub trait View {
    /// Render a VirtualNode, or any IntoIter<VirtualNode>
    fn render(&self) -> VirtualNode;
}
