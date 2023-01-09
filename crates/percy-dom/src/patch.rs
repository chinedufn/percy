//! Our Patch enum is intentionally kept in it's own file for easy inclusion into
//! The Percy Book.

use std::collections::{HashMap, HashSet};

pub use apply_patches::patch;

use crate::event::{EventHandler, EventName};
use crate::{AttributeValue, VText, VirtualNode};

mod apply_patches;

// TODO: pub(crate) BreadthFirstNodeIdx(pub u32);
type BreadthFirstNodeIdx = u32;

/// A Patch encodes an operation that modifies a real DOM element.
///
/// To update the real DOM that a user sees you'll want to first diff your
/// old virtual dom and new virtual dom.
///
/// This diff operation will generate `Vec<Patch>` with zero or more patches that, when
/// applied to your real DOM, will make your real DOM look like your new virtual dom.
///
/// Each Patch has a u32 node index that helps us identify the real DOM node that it applies to.
///
/// Our old virtual dom's nodes are indexed breadth first, as shown in this illustration
/// (0 being the root node, 1 being it's first child, 2 being it's second child).
///
/// ```text
///             .─.
///            ( 0 )
///             `┬'
///         ┌────┴──────┐
///         │           │
///         ▼           ▼
///        .─.         .─.
///       ( 1 )       ( 2 )
///        `┬'         `─'
///    ┌────┴───┐       │
///    │        │       ├─────┬─────┐
///    ▼        ▼       │     │     │
///   .─.      .─.      ▼     ▼     ▼
///  ( 3 )    ( 4 )    .─.   .─.   .─.
///   `─'      `─'    ( 5 ) ( 6 ) ( 7 )
///                    `─'   `─'   `─'
/// ```
///
/// The patching process is tested in a real browser in crates/percy-dom/tests/diff_patch.rs
// We use breadth-first traversal since it lets us know the indices of all of a node's siblings if we
// that node's index. We make use of this when diffing keyed lists.
// We haven't thought deeply through the implications of breadth first vs. depth first diffing.
// We can worry about that whenever we optimize our diffing and patching algorithms.
#[derive(Debug)]
#[cfg_attr(any(test, feature = "__test-utils"), derive(PartialEq))]
// TODO: Change all of these tuple structs with a `NodeIdx` to instead be `{old_idx: NodeIdx`} so
//  we can more easily tell which patches use the old node's index vs. the new one's.
pub enum Patch<'a> {
    /// Append a vector of child nodes to a parent node id.
    #[allow(missing_docs)]
    AppendChildren {
        parent_old_node_idx: BreadthFirstNodeIdx,
        new_nodes: Vec<&'a VirtualNode>,
    },
    /// Move the nodes to be the last of their parent's children.
    #[allow(missing_docs)]
    MoveToEndOfSiblings {
        parent_old_node_idx: BreadthFirstNodeIdx,
        siblings_to_move: Vec<BreadthFirstNodeIdx>,
    },
    /// Remove child nodes.
    RemoveChildren {
        /// The parent of the node that is being removed.
        parent_old_node_idx: BreadthFirstNodeIdx,
        /// The nodes to remove.
        to_remove: Vec<BreadthFirstNodeIdx>,
    },
    /// Replace a node with another node. This typically happens when a node's tag changes.
    /// ex: <div> becomes <span>
    #[allow(missing_docs)]
    Replace {
        old_idx: BreadthFirstNodeIdx,
        new_node: &'a VirtualNode,
    },
    /// Insert a new element before some other sibling element.
    #[allow(missing_docs)]
    InsertBefore {
        /// The node that isn't moving and is having another node inserted before it.
        anchor_old_node_idx: BreadthFirstNodeIdx,
        new_nodes: Vec<&'a VirtualNode>,
    },
    /// Move nodes to be before some other node.
    #[allow(missing_docs)]
    MoveNodesBefore {
        /// The node that we are moving other nodes to come before.
        /// This node is *NOT* moving during this patch.
        anchor_old_node_idx: BreadthFirstNodeIdx,
        /// The old node indices of the nodes that are being moved.
        to_move: Vec<BreadthFirstNodeIdx>,
    },
    /// The value attribute of a textarea or input element has not changed, but we will still patch
    /// it anyway in case something was typed into the field.
    ValueAttributeUnchanged(BreadthFirstNodeIdx, &'a AttributeValue),
    /// Add attributes that the new node has that the old node does not
    AddAttributes(BreadthFirstNodeIdx, HashMap<&'a str, &'a AttributeValue>),
    /// Remove attributes that the old node had that the new node doesn't
    RemoveAttributes(BreadthFirstNodeIdx, Vec<&'a str>),
    /// Change the text of a Text node.
    ChangeText(BreadthFirstNodeIdx, &'a VText),
    /// Patches that apply to [`SpecialAttributes`].
    SpecialAttribute(PatchSpecialAttribute<'a>),
    /// Insert events in the EventsByNodeIdx.
    /// If it is a non-delegated event the event will also get added to the DOM node.
    AddEvents(
        BreadthFirstNodeIdx,
        HashMap<&'a EventName, &'a EventHandler>,
    ),
    /// Remove events from the EventsByNodeIdx.
    /// If it is a non-delegated event the event will also get removed from the DOM node.
    RemoveEvents(BreadthFirstNodeIdx, Vec<(&'a EventName, &'a EventHandler)>),
    /// Delete all events in the EventsByNodeIdx for the given index, since the node has been
    /// removed from the DOM.
    RemoveAllVirtualEventsWithNodeIdx(BreadthFirstNodeIdx),
}

/// Patches that apply to [`SpecialAttributes`].
#[derive(Debug, PartialEq)]
pub enum PatchSpecialAttribute<'a> {
    /// Call the [`SpecialAttributes.on_create_elem`] function on the node.
    ///
    /// We only push this patch for existing nodes.
    /// New nodes get their on_create_element called automatically when they are created.
    CallOnCreateElemOnExistingNode(BreadthFirstNodeIdx, &'a VirtualNode),
    /// Call the [`SpecialAttributes.on_remove_elem`] function on the node.
    CallOnRemoveElem(BreadthFirstNodeIdx, &'a VirtualNode),
    /// Set the node's innerHTML using the [`SpecialAttributes.dangerous_inner_html`].
    SetDangerousInnerHtml(BreadthFirstNodeIdx, &'a VirtualNode),
    /// Set the node's innerHTML to an empty string.
    RemoveDangerousInnerHtml(BreadthFirstNodeIdx),
}

impl<'a> Patch<'a> {
    /// Every Patch is meant to be applied to a specific node within the DOM. Get the
    /// index of the DOM node that this patch should apply to. DOM nodes are indexed
    /// depth first with the root node in the tree having index 0.
    pub(crate) fn old_node_idx(&self) -> BreadthFirstNodeIdx {
        match self {
            Patch::AppendChildren {
                parent_old_node_idx: old_idx,
                ..
            } => *old_idx,
            Patch::Replace { old_idx, .. } => *old_idx,
            Patch::AddAttributes(node_idx, _) => *node_idx,
            Patch::RemoveAttributes(node_idx, _) => *node_idx,
            Patch::ChangeText(node_idx, _) => *node_idx,
            Patch::ValueAttributeUnchanged(node_idx, _) => *node_idx,
            Patch::SpecialAttribute(special) => match special {
                PatchSpecialAttribute::CallOnCreateElemOnExistingNode(node_idx, _) => *node_idx,
                PatchSpecialAttribute::SetDangerousInnerHtml(node_idx, _) => *node_idx,
                PatchSpecialAttribute::RemoveDangerousInnerHtml(node_idx) => *node_idx,
                PatchSpecialAttribute::CallOnRemoveElem(node_idx, _) => *node_idx,
            },
            Patch::RemoveEvents(node_idx, _) => *node_idx,
            Patch::AddEvents(node_idx, _) => *node_idx,
            Patch::RemoveAllVirtualEventsWithNodeIdx(node_idx) => {
                // TODO: We don't actually need the old node for this particular patch..
                //  so we should stop making use of this. Perhaps use `unreachable!()` and fix
                //  the places where we use this to stop calling this `.old_node_idx` function.
                *node_idx
            }
            Patch::InsertBefore {
                anchor_old_node_idx: old_idx,
                ..
            } => *old_idx,
            Patch::MoveNodesBefore {
                anchor_old_node_idx,
                ..
            } => *anchor_old_node_idx,
            Patch::MoveToEndOfSiblings {
                parent_old_node_idx,
                ..
            } => *parent_old_node_idx,
            Patch::RemoveChildren {
                parent_old_node_idx: parent_old_idx,
                ..
            } => *parent_old_idx,
        }
    }

    /// Insert the node indices that are needed in order to perform this patch.
    pub(crate) fn insert_node_indices_to_find(&self, to_find: &mut HashSet<u32>) {
        match self {
            Patch::AppendChildren {
                parent_old_node_idx: old_idx,
                ..
            } => {
                to_find.insert(*old_idx);
            }
            Patch::Replace { old_idx, .. } => {
                to_find.insert(*old_idx);
            }
            Patch::AddAttributes(node_idx, _) => {
                to_find.insert(*node_idx);
            }
            Patch::RemoveAttributes(node_idx, _) => {
                to_find.insert(*node_idx);
            }
            Patch::ChangeText(node_idx, _) => {
                to_find.insert(*node_idx);
            }
            Patch::ValueAttributeUnchanged(node_idx, _) => {
                to_find.insert(*node_idx);
            }
            Patch::SpecialAttribute(special) => match special {
                PatchSpecialAttribute::CallOnCreateElemOnExistingNode(node_idx, _) => {
                    to_find.insert(*node_idx);
                }
                PatchSpecialAttribute::SetDangerousInnerHtml(node_idx, _) => {
                    to_find.insert(*node_idx);
                }
                PatchSpecialAttribute::RemoveDangerousInnerHtml(node_idx) => {
                    to_find.insert(*node_idx);
                }
                PatchSpecialAttribute::CallOnRemoveElem(node_idx, _) => {
                    to_find.insert(*node_idx);
                }
            },
            Patch::RemoveEvents(node_idx, _) => {
                to_find.insert(*node_idx);
            }
            Patch::AddEvents(node_idx, _) => {
                to_find.insert(*node_idx);
            }
            Patch::RemoveAllVirtualEventsWithNodeIdx(node_idx) => {
                // TODO: We don't actually need the old node for this particular patch..
                //  so we should stop making use of this. Perhaps use `unreachable!()` and fix
                //  the places where we use this to stop calling this `.old_node_idx` function.
                to_find.insert(*node_idx);
            }
            Patch::InsertBefore {
                anchor_old_node_idx: old_idx,
                ..
            } => {
                to_find.insert(*old_idx);
            }
            Patch::MoveNodesBefore {
                anchor_old_node_idx,
                to_move,
            } => {
                to_find.insert(*anchor_old_node_idx);
                for idx in to_move {
                    to_find.insert(*idx);
                }
            }
            Patch::MoveToEndOfSiblings {
                parent_old_node_idx,
                siblings_to_move,
            } => {
                to_find.insert(*parent_old_node_idx);
                for idx in siblings_to_move {
                    to_find.insert(*idx);
                }
            }
            Patch::RemoveChildren {
                parent_old_node_idx,
                to_remove,
            } => {
                to_find.insert(*parent_old_node_idx);
                for idx in to_remove {
                    to_find.insert(*idx);
                }
            }
        }
    }
}
