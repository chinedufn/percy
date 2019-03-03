use crate::VirtualNode;

/// A trait with common functionality for rendering front-end views.
pub trait View {
    /// Render a VirtualNode
    ///
    /// FIXME: Return `IntoIter<VirtualNode` so that we can support views that return a vector
    /// of virtual nodes
    fn render(&self) -> VirtualNode;
}
