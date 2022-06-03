use crate::{View, VirtualNode};

/// Used by the html! macro for all braced child nodes so that we can use any type
/// that implements Into<IterableNodes>
///
/// html! { <div> { nodes } </div> }
///
/// nodes can be a String .. VirtualNode .. Vec<VirtualNode> ... etc
pub struct IterableNodes(Vec<VirtualNode>);

impl IterableNodes {
    /// Retrieve the first node mutably
    pub fn first_mut(&mut self) -> Option<&mut VirtualNode> {
        self.0.first_mut()
    }

    /// Retrieve the last node mutably
    pub fn last_mut(&mut self) -> Option<&mut VirtualNode> {
        self.0.last_mut()
    }
}

impl IntoIterator for IterableNodes {
    type Item = VirtualNode;
    // TODO: Is this possible with an array [VirtualNode] instead of a vec?
    type IntoIter = ::std::vec::IntoIter<VirtualNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<VirtualNode> for IterableNodes {
    fn from(other: VirtualNode) -> Self {
        IterableNodes(vec![other])
    }
}

impl From<&str> for IterableNodes {
    fn from(other: &str) -> Self {
        IterableNodes(vec![VirtualNode::text(other)])
    }
}

impl From<String> for IterableNodes {
    fn from(other: String) -> Self {
        IterableNodes(vec![VirtualNode::text(other.as_str())])
    }
}

impl From<&String> for IterableNodes {
    fn from(other: &String) -> Self {
        IterableNodes(vec![VirtualNode::text(other.as_str())])
    }
}

impl From<Vec<VirtualNode>> for IterableNodes {
    fn from(other: Vec<VirtualNode>) -> Self {
        IterableNodes(other)
    }
}

impl<V: View> From<V> for IterableNodes {
    fn from(from: V) -> Self {
        IterableNodes(vec![from.render()])
    }
}

impl<T: Into<IterableNodes>> From<Option<T>> for IterableNodes {
    fn from(opt: Option<T>) -> Self {
        if let Some(val) = opt {
            val.into()
        } else {
            IterableNodes(vec![])
        }
    }
}

impl<V: View> From<Vec<V>> for IterableNodes {
    fn from(other: Vec<V>) -> Self {
        IterableNodes(other.into_iter().map(|it| it.render()).collect())
    }
}

impl<V: View> From<&Vec<V>> for IterableNodes {
    fn from(other: &Vec<V>) -> Self {
        IterableNodes(other.iter().map(|it| it.render()).collect())
    }
}

impl<V: View> From<&[V]> for IterableNodes {
    fn from(other: &[V]) -> Self {
        IterableNodes(other.iter().map(|it| it.render()).collect())
    }
}

// Implements
//   From<T> and From<&T> -> IterableNodes
//   by using T's Display implementation.
macro_rules! from_display_impls {
    ($ty:ty) => {
        impl From<$ty> for IterableNodes {
            fn from(val: $ty) -> Self {
                IterableNodes::from(val.to_string())
            }
        }

        impl From<&$ty> for IterableNodes {
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
