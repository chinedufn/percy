use crate::event::RealDom;
use crate::{View, VirtualNode};

/// Used by the html! macro for all braced child nodes so that we can use any type
/// that implements Into<IterableNodes>
///
/// html! { <div> { nodes } </div> }
///
/// nodes can be a String .. VirtualNode .. Vec<VirtualNode> ... etc
pub struct IterableNodes<Handle: RealDom>(Vec<VirtualNode<Handle>>);

impl<Handle: RealDom> IterableNodes<Handle> {
    /// Retrieve the first node mutably
    pub fn first_mut(&mut self) -> Option<&mut VirtualNode<Handle>> {
        self.0.first_mut()
    }

    /// Retrieve the last node mutably
    pub fn last_mut(&mut self) -> Option<&mut VirtualNode<Handle>> {
        self.0.last_mut()
    }
}

impl<Handle: RealDom> IntoIterator for IterableNodes<Handle> {
    type Item = VirtualNode<Handle>;
    // TODO: Is this possible with an array [VirtualNode] instead of a vec?
    type IntoIter = ::std::vec::IntoIter<VirtualNode<Handle>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<Handle: RealDom> From<VirtualNode<Handle>> for IterableNodes<Handle> {
    fn from(other: VirtualNode<Handle>) -> Self {
        IterableNodes(vec![other])
    }
}

impl<Handle: RealDom> From<&str> for IterableNodes<Handle> {
    fn from(other: &str) -> Self {
        IterableNodes(vec![VirtualNode::new_text(other)])
    }
}

impl<Handle: RealDom> From<String> for IterableNodes<Handle> {
    fn from(other: String) -> Self {
        IterableNodes(vec![VirtualNode::new_text(other.as_str())])
    }
}

impl<Handle: RealDom> From<&String> for IterableNodes<Handle> {
    fn from(other: &String) -> Self {
        IterableNodes(vec![VirtualNode::new_text(other.as_str())])
    }
}

impl<Handle: RealDom> From<Vec<VirtualNode<Handle>>> for IterableNodes<Handle> {
    fn from(other: Vec<VirtualNode<Handle>>) -> Self {
        IterableNodes(other)
    }
}

#[cfg(feature = "web")]
impl<V: View<web_sys::Window>> From<V> for IterableNodes<web_sys::Window> {
    fn from(from: V) -> Self {
        IterableNodes(vec![from.render()])
    }
}

impl<T: Into<IterableNodes<Handle>>, Handle: RealDom> From<Option<T>> for IterableNodes<Handle> {
    fn from(opt: Option<T>) -> Self {
        if let Some(val) = opt {
            val.into()
        } else {
            IterableNodes(vec![])
        }
    }
}

#[cfg(feature = "web")]
impl<V: View<web_sys::Window>> From<Vec<V>> for IterableNodes<web_sys::Window> {
    fn from(other: Vec<V>) -> Self {
        IterableNodes(other.into_iter().map(|it| it.render()).collect())
    }
}

impl<V: View<Handle>, Handle: RealDom> From<&Vec<V>> for IterableNodes<Handle> {
    fn from(other: &Vec<V>) -> Self {
        IterableNodes(other.iter().map(|it| it.render()).collect())
    }
}

impl<V: View<Handle>, Handle: RealDom> From<&[V]> for IterableNodes<Handle> {
    fn from(other: &[V]) -> Self {
        IterableNodes(other.iter().map(|it| it.render()).collect())
    }
}

// Implements
//   From<T> and From<&T> -> IterableNodes
//   by using T's Display implementation.
macro_rules! from_display_impls {
    ($ty:ty) => {
        impl<Handle: RealDom> From<$ty> for IterableNodes<Handle> {
            fn from(val: $ty) -> Self {
                IterableNodes::from(val.to_string())
            }
        }

        impl<Handle: RealDom> From<&$ty> for IterableNodes<Handle> {
            fn from(val: &$ty) -> Self {
                IterableNodes::from(val.to_string())
            }
        }
    };

    ($ty:ty, $($tys:ty),*) => {
        from_display_impls!( $ty );
        from_display_impls! ( $($tys),* );
    }
}
from_display_impls!(u8, u16, u32, usize, u64, u128, i8, i16, i32, isize, i64, i128, f32, f64);
