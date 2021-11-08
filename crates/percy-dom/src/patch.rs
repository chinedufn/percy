//! Our Patch enum is intentionally kept in it's own file for easy inclusion into
//! The Percy Book.

use std::collections::HashMap;

pub use apply_patches::patch;

use crate::event::{EventHandler, EventName};
use crate::{AttributeValue, VText, VirtualNode};

mod apply_patches;

type NodeIdx = u32;

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
/// Our old virtual dom's nodes are indexed depth first, as shown in this illustration
/// (0 being the root node, 1 being it's first child, 2 being it's first child's first child).
///
/// ```text
///             .─.
///            ( 0 )
///             `┬'
///         ┌────┴──────┐
///         │           │
///         ▼           ▼
///        .─.         .─.
///       ( 1 )       ( 4 )
///        `┬'         `─'
///    ┌────┴───┐       │
///    │        │       ├─────┬─────┐
///    ▼        ▼       │     │     │
///   .─.      .─.      ▼     ▼     ▼
///  ( 2 )    ( 3 )    .─.   .─.   .─.
///   `─'      `─'    ( 5 ) ( 6 ) ( 7 )
///                    `─'   `─'   `─'
/// ```
///
/// The patching process is tested in a real browser in crates/percy-dom/tests/diff_patch.rs
#[derive(Debug)]
#[cfg_attr(any(test, feature = "__test-utils"), derive(PartialEq))]
// TODO: Change all of these tuple structs with a `NodeIdx` to instead be `{old_idx: NodeIdx`} so
//  we can more easily tell which patches use the old node's index vs. the new one's.
pub enum Patch<'a> {
    /// Append a vector of child nodes to a parent node id.
    #[allow(missing_docs)]
    AppendChildren {
        old_idx: NodeIdx,
        new_nodes: Vec<(NodeIdx, &'a VirtualNode)>,
    },
    /// For a `node_i32`, remove all children besides the first `len`
    TruncateChildren(NodeIdx, usize),
    /// Replace a node with another node. This typically happens when a node's tag changes.
    /// ex: <div> becomes <span>
    #[allow(missing_docs)]
    Replace {
        old_idx: NodeIdx,
        new_idx: NodeIdx,
        new_node: &'a VirtualNode,
    },
    /// The value attribute of a textarea or input element has not changed, but we will still patch
    /// it anyway in case something was typed into the field.
    ValueAttributeUnchanged(NodeIdx, &'a AttributeValue),
    /// Add attributes that the new node has that the old node does not
    AddAttributes(NodeIdx, HashMap<&'a str, &'a AttributeValue>),
    /// Remove attributes that the old node had that the new node doesn't
    RemoveAttributes(NodeIdx, Vec<&'a str>),
    /// Change the text of a Text node.
    ChangeText(NodeIdx, &'a VText),
    /// Patches that apply to [`SpecialAttributes`].
    SpecialAttribute(PatchSpecialAttribute<'a>),
    /// Remove the `__events_id__` property from the DOM node.
    /// This happens when the node no longer has any events.
    RemoveEventsId(NodeIdx),
    /// Set the `__events_id__` property on the DOM node.
    #[allow(missing_docs)]
    SetEventsId { old_idx: NodeIdx, new_idx: NodeIdx },
    /// Insert events in the EventsByNodeIdx.
    /// If it is a non-delegated event the event will also get added to the DOM node.
    AddEvents(NodeIdx, HashMap<&'a EventName, &'a EventHandler>),
    /// Remove events from the EventsByNodeIdx.
    /// If it is a non-delegated event the event will also get removed from the DOM node.
    RemoveEvents(NodeIdx, Vec<(&'a EventName, &'a EventHandler)>),
    /// Delete all events in the EventsByNodeIdx for the given index, since the node has been
    /// removed from the DOM.
    RemoveAllManagedEventsWithNodeIdx(NodeIdx),
}

/// Patches that apply to [`SpecialAttributes`].
#[derive(Debug, PartialEq)]
pub enum PatchSpecialAttribute<'a> {
    /// Call the [`SpecialAttributes.on_create_elem`] function on the node.
    CallOnCreateElem(NodeIdx, &'a VirtualNode),
    /// Call the [`SpecialAttributes.on_create_elem`] function on the node.
    CallOnRemoveElem(NodeIdx, &'a VirtualNode),
    /// Set the node's innerHTML using the [`SpecialAttributes.dangerous_inner_html`].
    SetDangerousInnerHtml(NodeIdx, &'a VirtualNode),
    /// Set the node's innerHTML to an empty string.
    RemoveDangerousInnerHtml(NodeIdx),
}

impl<'a> Patch<'a> {
    /// Every Patch is meant to be applied to a specific node within the DOM. Get the
    /// index of the DOM node that this patch should apply to. DOM nodes are indexed
    /// depth first with the root node in the tree having index 0.
    pub(crate) fn old_node_idx(&self) -> NodeIdx {
        match self {
            Patch::AppendChildren { old_idx, .. } => *old_idx,
            Patch::TruncateChildren(node_idx, _) => *node_idx,
            Patch::Replace { old_idx, .. } => *old_idx,
            Patch::AddAttributes(node_idx, _) => *node_idx,
            Patch::RemoveAttributes(node_idx, _) => *node_idx,
            Patch::ChangeText(node_idx, _) => *node_idx,
            Patch::ValueAttributeUnchanged(node_idx, _) => *node_idx,
            Patch::SpecialAttribute(special) => match special {
                PatchSpecialAttribute::CallOnCreateElem(node_idx, _) => *node_idx,
                PatchSpecialAttribute::SetDangerousInnerHtml(node_idx, _) => *node_idx,
                PatchSpecialAttribute::RemoveDangerousInnerHtml(node_idx) => *node_idx,
                PatchSpecialAttribute::CallOnRemoveElem(node_idx, _) => *node_idx,
            },
            Patch::RemoveEventsId(node_idx) => *node_idx,
            Patch::SetEventsId { old_idx, .. } => *old_idx,
            Patch::RemoveEvents(node_idx, _) => *node_idx,
            Patch::AddEvents(node_idx, _) => *node_idx,
            Patch::RemoveAllManagedEventsWithNodeIdx(node_idx) => {
                // TODO: We don't actually need the old node for this particular patch..
                //  so we should stop making use of this. Perhaps use `unreachable!()` and fix
                //  the places where we use this to stop using it.
                *node_idx
            }
        }
    }
}
