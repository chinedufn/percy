use crate::diff::longest_increasing_subsequence::{
    get_longest_increasing_subsequence, KeyAndChildIdx,
};
use crate::event::{EventHandler, EventName};
use crate::{AttributeValue, Patch, PatchSpecialAttribute};
use crate::{VElement, VirtualNode};
use std::collections::{HashMap, VecDeque};
use std::mem;

mod longest_increasing_subsequence;

/// Given two VirtualNode's generate Patch's that would turn the old virtual node's
/// real DOM node equivalent into the new VirtualNode's real DOM node equivalent.
pub fn diff<'a>(old: &'a VirtualNode, new: &'a VirtualNode) -> Vec<Patch<'a>> {
    let old_node_idx = 0;
    let mut diff_queue: VecDeque<DiffJob> = VecDeque::new();

    diff_queue.push_back(DiffJob {
        old_node_idx,
        old,
        new,
    });

    let mut ctx = DiffContext::new(old, new);
    diff_recursive(&mut ctx);

    ctx.take_patches()
}

enum Job<'a> {
    Diff(DiffJob<'a>),
    ProcessDeleted(DeleteJob<'a>),
}

#[derive(Copy, Clone)]
struct DiffJob<'a> {
    // The old node's breadth-first index in the old tree.
    old_node_idx: u32,
    old: &'a VirtualNode,
    new: &'a VirtualNode,
}

struct DeleteJob<'a> {
    old_node_idx: u32,
    old: &'a VirtualNode,
}

use self::diff_ctx::DiffContext;
mod diff_ctx {
    use super::*;

    // Kept in its own module in order to keep fields private so that we can enforce the
    // DiffContext's API.
    pub(super) struct DiffContext<'a> {
        job_queue: VecDeque<Job<'a>>,
        patches: Vec<Patch<'a>>,
        next_old_node_idx: u32,
    }
    impl<'a> DiffContext<'a> {
        pub fn new(old: &'a VirtualNode, new: &'a VirtualNode) -> Self {
            let mut job_queue = VecDeque::new();
            job_queue.push_back(Job::Diff(DiffJob {
                old_node_idx: 0,
                old,
                new,
            }));

            Self {
                job_queue,
                patches: Vec::new(),
                next_old_node_idx: 1,
            }
        }

        pub fn next_job(&mut self) -> Option<Job<'a>> {
            self.job_queue.pop_front()
        }

        pub fn push_diff_job(&mut self, diff_job: DiffJob<'a>) {
            self.job_queue.push_back(Job::Diff(diff_job))
        }

        pub fn push_delete_job(&mut self, delete_job: DeleteJob<'a>) {
            self.job_queue.push_back(Job::ProcessDeleted(delete_job))
        }

        /// We always push patches if the node has not been deleted.
        /// If the node or one of its ancestors has been deleted we only push patches that are
        /// applicable to deleted nodes.
        pub fn push_patch(&mut self, patch: Patch<'a>) {
            self.patches.push(patch);
        }

        pub fn next_old_node_idx(&self) -> u32 {
            self.next_old_node_idx
        }

        pub fn increment_old_node_idx(&mut self, increment: usize) {
            self.next_old_node_idx += increment as u32;
        }

        pub fn take_patches(self) -> Vec<Patch<'a>> {
            self.patches
        }
    }
}

fn diff_recursive(ctx: &mut DiffContext) {
    let job = ctx.next_job();
    match job {
        Some(Job::Diff(diff_job)) => process_diff_job(ctx, diff_job),
        Some(Job::ProcessDeleted(delete_job)) => process_delete_job(ctx, delete_job),
        None => return,
    };

    diff_recursive(ctx);
}

fn process_diff_job<'a>(ctx: &mut DiffContext<'a>, diff_job: DiffJob<'a>) {
    let old = diff_job.old;
    let new = diff_job.new;
    let old_node_idx = diff_job.old_node_idx;

    let node_variants_different = mem::discriminant(old) != mem::discriminant(new);
    let mut element_tags_different = false;

    if let (VirtualNode::Element(old_element), VirtualNode::Element(new_element)) = (old, new) {
        element_tags_different = old_element.tag != new_element.tag;
    }

    let should_fully_replace_node = node_variants_different || element_tags_different;

    if should_fully_replace_node {
        replace_node(diff_job, ctx);
        return;
    }

    match (old, new) {
        (VirtualNode::Text(old_text), VirtualNode::Text(new_text)) => {
            if old_text != new_text {
                ctx.push_patch(Patch::ChangeText(old_node_idx, &new_text));
            }
        }

        (VirtualNode::Element(old_element), VirtualNode::Element(new_element)) => {
            let mut attributes_to_add: HashMap<&str, &AttributeValue> = HashMap::new();
            let mut attributes_to_remove: Vec<&str> = vec![];

            let mut events_to_add = HashMap::new();
            let mut events_to_remove = vec![];

            find_attributes_to_add(
                old_node_idx,
                &mut attributes_to_add,
                old_element,
                new_element,
                ctx,
            );

            find_attributes_to_remove(
                &mut attributes_to_add,
                &mut attributes_to_remove,
                old_element,
                new_element,
            );

            find_events_to_add(&mut events_to_add, old_element, new_element);
            find_events_to_remove(
                &mut events_to_add,
                &mut events_to_remove,
                old_element,
                new_element,
            );

            if attributes_to_add.len() > 0 {
                ctx.push_patch(Patch::AddAttributes(old_node_idx, attributes_to_add));
            }
            if attributes_to_remove.len() > 0 {
                ctx.push_patch(Patch::RemoveAttributes(old_node_idx, attributes_to_remove));
            }

            if events_to_remove.len() > 0 {
                ctx.push_patch(Patch::RemoveEvents(old_node_idx, events_to_remove));
            }
            if events_to_add.len() > 0 {
                ctx.push_patch(Patch::AddEvents(old_node_idx, events_to_add));
            }

            maybe_push_inner_html_patch(new, old_element, new_element, old_node_idx, ctx);

            maybe_push_on_create_element_patch(new, old_element, new_element, old_node_idx, ctx);
            maybe_push_on_remove_element_patch(old, old_element, new_element, old_node_idx, ctx);

            generate_patches_for_children(old_node_idx, old_element, new_element, ctx);
        }
        (VirtualNode::Text(_), VirtualNode::Element(_))
        | (VirtualNode::Element(_), VirtualNode::Text(_)) => {
            unreachable!("Unequal variant discriminants should already have been handled");
        }
    };
}

fn process_delete_job<'a>(ctx: &mut DiffContext<'a>, delete_job: DeleteJob<'a>) {
    if let VirtualNode::Element(element_node) = delete_job.old {
        if element_node.events.len() > 0 {
            ctx.push_patch(Patch::RemoveAllVirtualEventsWithNodeIdx(
                delete_job.old_node_idx,
            ));
        }

        if element_node
            .special_attributes
            .on_remove_element_key()
            .is_some()
        {
            ctx.push_patch(Patch::SpecialAttribute(
                PatchSpecialAttribute::CallOnRemoveElem(delete_job.old_node_idx, delete_job.old),
            ));
        }

        maybe_push_delete_jobs_for_children(ctx, delete_job.old);
    }
}

/// Push patches for replacing a node.
fn replace_node<'a>(diff_job: DiffJob<'a>, ctx: &mut DiffContext<'a>) {
    if let Some(elem) = diff_job.old.as_velement_ref() {
        if elem.events.has_events() {
            ctx.push_patch(Patch::RemoveAllVirtualEventsWithNodeIdx(
                diff_job.old_node_idx,
            ));
        }
    }

    match diff_job.old.as_velement_ref() {
        Some(elem) if elem.special_attributes.on_remove_element_key().is_some() => {
            ctx.push_patch(Patch::SpecialAttribute(
                PatchSpecialAttribute::CallOnRemoveElem(diff_job.old_node_idx, diff_job.old),
            ));
        }
        _ => {}
    };
    ctx.push_patch(Patch::Replace {
        old_idx: diff_job.old_node_idx,
        new_node: diff_job.new,
    });

    maybe_push_delete_jobs_for_children(ctx, diff_job.old);
}

fn maybe_push_delete_jobs_for_children<'a>(ctx: &mut DiffContext<'a>, node: &'a VirtualNode) {
    if let VirtualNode::Element(old_element_node) = node {
        let node_idx_of_first_child = ctx.next_old_node_idx();
        ctx.increment_old_node_idx(old_element_node.children.len());

        for (idx, child) in old_element_node.children.iter().enumerate() {
            let cur_node_idx = node_idx_of_first_child + idx as u32;
            ctx.push_delete_job(DeleteJob {
                old_node_idx: cur_node_idx,
                old: &child,
            });
        }
    }
}

/// Add attributes from the new element that are not already on the old one or that have changed.
fn find_attributes_to_add<'a>(
    cur_node_idx: u32,
    attributes_to_add: &mut HashMap<&'a str, &'a AttributeValue>,
    old_element: &VElement,
    new_element: &'a VElement,
    ctx: &mut DiffContext<'a>,
) {
    for (new_attr_name, new_attr_val) in new_element.attrs.iter() {
        if new_attr_name == "key" {
            continue;
        }

        match old_element.attrs.get(new_attr_name) {
            Some(ref old_attr_val) => {
                if old_attr_val != &new_attr_val {
                    attributes_to_add.insert(new_attr_name, new_attr_val);
                } else if new_attr_name == "value" {
                    ctx.push_patch(Patch::ValueAttributeUnchanged(cur_node_idx, new_attr_val));
                }
            }
            None => {
                attributes_to_add.insert(new_attr_name, new_attr_val);
            }
        };
    }
}

/// Remove attributes that were on the old element that are not present on the new element.
fn find_attributes_to_remove<'a>(
    attributes_to_add: &mut HashMap<&str, &AttributeValue>,
    attributes_to_remove: &mut Vec<&'a str>,
    old_element: &'a VElement,
    new_element: &VElement,
) {
    for (old_attr_name, old_attr_val) in old_element.attrs.iter() {
        if old_attr_name == "key" {
            continue;
        }

        if attributes_to_add.get(&old_attr_name[..]).is_some() {
            continue;
        };

        match new_element.attrs.get(old_attr_name) {
            Some(ref new_attr_val) => {
                if new_attr_val != &old_attr_val {
                    attributes_to_remove.push(old_attr_name);
                }
            }
            None => {
                attributes_to_remove.push(old_attr_name);
            }
        };
    }
}

/// Add attributes from the new element that are not already on the old one or that have changed.
fn find_events_to_add<'a>(
    events_to_add: &mut HashMap<&'a EventName, &'a EventHandler>,
    old_element: &VElement,
    new_element: &'a VElement,
) {
    for (new_event_name, new_event) in new_element.events.iter() {
        if !old_element.events.contains_key(new_event_name) {
            events_to_add.insert(new_event_name, new_event);
        }
    }
}

/// Remove non delegated that were on the old element that are not present on the new element.
fn find_events_to_remove<'a>(
    events_to_add: &mut HashMap<&'a EventName, &'a EventHandler>,
    events_to_remove: &mut Vec<(&'a EventName, &'a EventHandler)>,
    old_element: &'a VElement,
    new_element: &'a VElement,
) {
    for (old_event_name, old_event) in old_element.events.iter() {
        if events_to_add.contains_key(old_event_name) {
            continue;
        };
        if new_element.events.contains_key(old_event_name) {
            continue;
        }

        events_to_remove.push((old_event_name, old_event));
    }
}

fn maybe_push_inner_html_patch<'a>(
    new: &'a VirtualNode,
    old_element: &VElement,
    new_element: &VElement,
    old_node_idx: u32,
    ctx: &mut DiffContext<'a>,
) {
    match (
        old_element.special_attributes.dangerous_inner_html.as_ref(),
        new_element.special_attributes.dangerous_inner_html.as_ref(),
    ) {
        (None, Some(_)) => {
            ctx.push_patch(Patch::SpecialAttribute(
                PatchSpecialAttribute::SetDangerousInnerHtml(old_node_idx, new),
            ));
        }
        (Some(old_inner), Some(new_inner)) => {
            if old_inner != new_inner {
                ctx.push_patch(Patch::SpecialAttribute(
                    PatchSpecialAttribute::SetDangerousInnerHtml(old_node_idx, new),
                ));
            }
        }
        (Some(_), None) => {
            ctx.push_patch(Patch::SpecialAttribute(
                PatchSpecialAttribute::RemoveDangerousInnerHtml(old_node_idx),
            ));
        }
        (None, None) => {}
    };
}

fn maybe_push_on_create_element_patch<'a>(
    new: &'a VirtualNode,
    old_element: &VElement,
    new_element: &VElement,
    old_node_idx: u32,
    ctx: &mut DiffContext<'a>,
) {
    match (
        old_element.special_attributes.on_create_element_key(),
        new_element.special_attributes.on_create_element_key(),
    ) {
        (None, Some(_)) => {
            ctx.push_patch(Patch::SpecialAttribute(
                PatchSpecialAttribute::CallOnCreateElemOnExistingNode(old_node_idx, new),
            ));
        }
        (Some(old_id), Some(new_id)) => {
            if new_id != old_id {
                ctx.push_patch(Patch::SpecialAttribute(
                    PatchSpecialAttribute::CallOnCreateElemOnExistingNode(old_node_idx, new),
                ));
            }
        }
        (Some(_), None) | (None, None) => {}
    };
}

fn maybe_push_on_remove_element_patch<'a>(
    old: &'a VirtualNode,
    old_element: &VElement,
    new_element: &VElement,
    old_node_idx: u32,
    ctx: &mut DiffContext<'a>,
) {
    let old_on_remove_elem_key = old_element.special_attributes.on_remove_element_key();

    let should_call_on_remove_elem = match (
        old_on_remove_elem_key,
        new_element.special_attributes.on_remove_element_key(),
    ) {
        (Some(_), None) => true,
        (Some(old_id), Some(new_id)) => old_id != new_id,
        _ => false,
    };
    if should_call_on_remove_elem {
        ctx.push_patch(Patch::SpecialAttribute(
            PatchSpecialAttribute::CallOnRemoveElem(old_node_idx, old),
        ));
    }
}

fn generate_patches_for_children<'a, 'b>(
    parent_old_node_idx: u32,
    old_element: &'a VElement,
    new_element: &'a VElement,
    ctx: &mut DiffContext<'a>,
) {
    // TODO: Refactor into smaller functions
    //       Optimize

    let old_child_count = old_element.children.len();

    let mut key_to_old_child_idx: HashMap<ElementKey, usize> = HashMap::new();
    let mut key_to_new_child_idx: HashMap<ElementKey, usize> = HashMap::new();

    let mut old_non_keyed_and_no_longer_keyed_nodes: VecDeque<usize> = Default::default();

    let mut new_node_keys: HashMap<usize, ElementKey> = HashMap::new();

    let mut old_tracked_indices = TrackedImplicitlyKeyableIndices::default();
    let mut new_tracked_indices = TrackedImplicitlyKeyableIndices::default();

    for (idx, new_child) in new_element.children.iter().enumerate() {
        let implicit_key = new_tracked_indices.get_key_maybe_increment(new_child);
        let new_key = node_key(new_child, implicit_key);

        if let Some(new_key) = new_key {
            new_node_keys.insert(idx, new_key);
            key_to_new_child_idx.insert(new_key, idx);
        }
    }

    let node_idx_of_first_child = ctx.next_old_node_idx();
    ctx.increment_old_node_idx(old_element.children.len());

    for (idx, old_child) in old_element.children.iter().enumerate() {
        let implicit_key = old_tracked_indices.get_key_maybe_increment(old_child);
        let old_key = node_key(old_child, implicit_key);

        match old_key {
            Some(old_key) if key_to_new_child_idx.contains_key(&old_key) => {
                key_to_old_child_idx.insert(old_key, idx);
            }
            _ => {
                old_non_keyed_and_no_longer_keyed_nodes.push_back(idx);
            }
        }
    }

    let mut old_child_indices_of_preserved_keys = vec![];

    for new_child_idx in 0..new_element.children.len() {
        let key = new_node_keys.get(&new_child_idx);
        let Some(key) = key else {
            continue;
        };

        let old_key_child_idx = key_to_old_child_idx.get(key);
        let old_key_child_idx = if let Some(k) = old_key_child_idx {
            k
        } else {
            continue;
        };

        old_child_indices_of_preserved_keys.push(KeyAndChildIdx {
            key: *key,
            child_idx: *old_key_child_idx,
        });
    }

    let longest_increasing: HashMap<ElementKey, usize> =
        get_longest_increasing_subsequence(&old_child_indices_of_preserved_keys)
            .into_iter()
            .map(|k| (k.key, k.child_idx))
            .collect();

    enum InsertBeforeOrMoveBefore<'a> {
        InsertBefore(&'a VirtualNode),
        MoveBefore(u32),
    }
    enum PlaceBeforeKind {
        Insert,
        Move,
    }

    let mut insert_before = vec![];
    let mut move_before = vec![];

    let mut insert_before_or_move = vec![];

    // (Child idx, DiffJob)
    let mut jobs: Vec<(usize, DiffJob)> = vec![];

    let mut new_tracked_indices = TrackedImplicitlyKeyableIndices::default();
    for (new_child_idx, new_child_node) in new_element.children.iter().enumerate() {
        let implicit_key = new_tracked_indices.get_key_maybe_increment(new_child_node);
        let key = node_key(new_child_node, implicit_key);
        let old_child_idx = key.as_ref().and_then(|key| longest_increasing.get(key));

        match old_child_idx {
            Some(old_child_idx) => {
                let old_idx = node_idx_of_first_child + *old_child_idx as u32;

                let mut previous = None;

                for insert_or_move in &insert_before_or_move {
                    match insert_or_move {
                        InsertBeforeOrMoveBefore::InsertBefore(insert) => {
                            if matches!(previous, Some(PlaceBeforeKind::Move)) {
                                maybe_push_move_before(ctx, old_idx, &mut move_before);
                            }

                            insert_before.push(*insert);
                            previous = Some(PlaceBeforeKind::Insert);
                        }
                        InsertBeforeOrMoveBefore::MoveBefore(m) => {
                            if matches!(previous, Some(PlaceBeforeKind::Insert)) {
                                maybe_push_insert_before(ctx, old_idx, &mut insert_before);
                            }

                            move_before.push(*m);
                            previous = Some(PlaceBeforeKind::Move);
                        }
                    }
                }

                insert_before_or_move.clear();
                maybe_push_insert_before(ctx, old_idx, &mut insert_before);
                maybe_push_move_before(ctx, old_idx, &mut move_before);

                let old_idx = node_idx_of_first_child + *old_child_idx as u32;

                let job = DiffJob {
                    old_node_idx: old_idx,
                    old: &old_element.children[*old_child_idx],
                    new: new_child_node,
                };
                jobs.push((*old_child_idx, job));
            }
            None => {
                let old_child_idx = key.and_then(|key| key_to_old_child_idx.get(&key));

                if let Some(old_child_idx) = old_child_idx {
                    let old_idx = parent_old_node_idx + 1 + *old_child_idx as u32;

                    insert_before_or_move.push(InsertBeforeOrMoveBefore::MoveBefore(old_idx));

                    let job = DiffJob {
                        old_node_idx: old_idx,
                        old: &old_element.children[*old_child_idx],
                        new: new_child_node,
                    };
                    jobs.push((*old_child_idx, job));
                } else {
                    if let Some(old_child_idx) = old_non_keyed_and_no_longer_keyed_nodes.pop_front()
                    {
                        let old_idx = node_idx_of_first_child + old_child_idx as u32;

                        if old_child_idx != new_child_idx {
                            insert_before_or_move
                                .push(InsertBeforeOrMoveBefore::MoveBefore(old_idx));
                        }

                        let job = DiffJob {
                            old_node_idx: old_idx,
                            old: &old_element.children[old_child_idx],
                            new: new_child_node,
                        };
                        jobs.push((old_child_idx, job));
                    } else {
                        insert_before_or_move
                            .push(InsertBeforeOrMoveBefore::InsertBefore(new_child_node));
                    }
                }
            }
        }
    }

    jobs.sort_by(|a, b| a.0.cmp(&b.0));

    let mut to_remove = vec![];
    for child_idx in 0..old_child_count {
        if jobs.get(0).map(|j| j.0) == Some(child_idx) {
            let job = jobs.remove(0);
            ctx.push_diff_job(job.1);
        } else {
            let node_idx = node_idx_of_first_child + child_idx as u32;

            to_remove.push(node_idx);

            ctx.push_delete_job(DeleteJob {
                old_node_idx: node_idx,
                old: &old_element.children[child_idx],
            });
        }
    }

    if to_remove.len() > 0 {
        ctx.push_patch(Patch::RemoveChildren {
            parent_old_node_idx,
            to_remove,
        });
    }

    let mut appends = vec![];
    let mut moves = vec![];
    let mut previous = None;

    for item in insert_before_or_move {
        match item {
            InsertBeforeOrMoveBefore::InsertBefore(new_node) => {
                if matches!(previous, Some(PlaceBeforeKind::Move)) {
                    ctx.push_patch(Patch::MoveToEndOfSiblings {
                        parent_old_node_idx,
                        siblings_to_move: moves.clone(),
                    });
                    moves.clear();
                }

                appends.push(new_node);
                previous = Some(PlaceBeforeKind::Insert);
            }
            InsertBeforeOrMoveBefore::MoveBefore(m) => {
                if matches!(previous, Some(PlaceBeforeKind::Insert)) {
                    ctx.push_patch(Patch::AppendChildren {
                        parent_old_node_idx,
                        new_nodes: appends.clone(),
                    });
                    appends.clear();
                }

                moves.push(m);
                previous = Some(PlaceBeforeKind::Move);
            }
        }
    }

    if appends.len() >= 1 {
        ctx.push_patch(Patch::AppendChildren {
            parent_old_node_idx,
            new_nodes: appends.clone(),
        });
    } else if moves.len() >= 1 {
        ctx.push_patch(Patch::MoveToEndOfSiblings {
            parent_old_node_idx,
            siblings_to_move: moves.clone(),
        });
    }
}

fn maybe_push_insert_before<'a>(
    ctx: &mut DiffContext<'a>,
    old_idx: u32,
    new_nodes: &mut Vec<&'a VirtualNode>,
) {
    if new_nodes.len() == 0 {
        return;
    }

    ctx.push_patch(Patch::InsertBefore {
        anchor_old_node_idx: old_idx,
        new_nodes: new_nodes.clone(),
    });
    new_nodes.clear();
}

fn maybe_push_move_before<'a>(ctx: &mut DiffContext<'a>, old_idx: u32, move_before: &mut Vec<u32>) {
    if move_before.len() == 0 {
        return;
    }

    ctx.push_patch(Patch::MoveNodesBefore {
        anchor_old_node_idx: old_idx,
        to_move: move_before.clone(),
    });
    move_before.clear();
}

fn node_key(
    node: &VirtualNode,
    implicit_elem_key: Option<ElementKeyImplicit>,
) -> Option<ElementKey> {
    let elem = node.as_velement_ref()?;

    let explicit_key = elem
        .attrs
        .get("key")
        .and_then(|k| k.as_string().map(String::as_str))
        .map(ElementKey::Explicit);

    if explicit_key.is_some() {
        return explicit_key;
    }

    implicit_elem_key.map(ElementKey::Implicit)
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(test, derive(Debug))]
enum ElementKey<'a> {
    /// An explicit key set using the `key = "..."` attribute.
    Explicit(&'a str),
    Implicit(ElementKeyImplicit),
}

/// Implicit keys that we used for certain elements.
/// We do not use an implicit key if the element has an explicit key.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(test, derive(Debug))]
enum ElementKeyImplicit {
    /// An implicit key that we use for focusable elements.
    /// This ensures that, for example, when sibling nodes ar prepending these elements get moved
    /// instead of deleted and recreated, meaning that if the element is currently focused in the
    /// real DOM it won't lose its focus.
    Focusable { focusable_idx: FocusableIdx },
}

/// The nodes index across all of its focusable siblings of the same tag.
/// So, the first input element sibling has index 0, then the next input element has index 1, etc.
/// If the siblings are "input, div, div, textarea, textarea", the input's `focusable_idx` would be 0
/// and the textareas' `focusable_idx`s would be 0 and 1.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(test, derive(Debug))]
enum FocusableIdx {
    Input(usize),
    TextArea(usize),
}

#[derive(Default)]
struct TrackedImplicitlyKeyableIndices {
    input: usize,
    textarea: usize,
}

impl TrackedImplicitlyKeyableIndices {
    /// If the element has an implicit key return it.
    pub fn get_key_maybe_increment(&mut self, node: &VirtualNode) -> Option<ElementKeyImplicit> {
        let elem = node.as_velement_ref()?;
        let key = match elem.tag.as_str() {
            "input" => {
                let old_idx = self.input;
                self.input += 1;
                ElementKeyImplicit::Focusable {
                    focusable_idx: FocusableIdx::Input(old_idx),
                }
            }
            "textarea" => {
                let old_idx = self.input;
                self.textarea += 1;
                ElementKeyImplicit::Focusable {
                    focusable_idx: FocusableIdx::TextArea(old_idx),
                }
            }
            _ => None?,
        };
        Some(key)
    }
}

// Kept in it's own file so that we can import it into the Percy book without extra indentation.
#[cfg(test)]
mod diff_test_case;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventName;
    use crate::{html, EventAttribFn, PatchSpecialAttribute, VText, VirtualNode};
    use std::collections::HashMap;
    use std::rc::Rc;
    use virtual_node::IterableNodes;
    use wasm_bindgen::JsValue;

    use super::diff_test_case::*;

    /// Verify that we can generate patches that replace a virtual node with another one.
    #[test]
    fn replace_node() {
        DiffTestCase {
            old: html! { <div> </div> },
            new: html! { <span> </span> },
            expected: vec![Patch::Replace {
                old_idx: 0,
                new_node: &html! { <span></span> },
            }],
        }
        .test();
        DiffTestCase {
            old: html! { <div> <b></b> </div> },
            new: html! { <div> <strong></strong> </div> },
            expected: vec![Patch::Replace {
                old_idx: 1,
                new_node: &html! { <strong></strong> },
            }],
        }
        .test();
        DiffTestCase {
            old: html! { <div> <b>1</b> <em></em> </div> },
            new: html! { <div> <i>{"1"} {"2"}</i> <br /> </div>},
            expected: vec![
                Patch::Replace {
                    old_idx: 1,
                    new_node: &html! { <i>{"1"} {"2"}</i> },
                },
                Patch::Replace {
                    old_idx: 2,
                    new_node: &html! { <br /> },
                },
            ],
        }
        .test();

        DiffTestCase {
            old: html! {
              // 0
              <div>
                  // 1
                  <span>
                      // 3
                      <div>
                          // 5 (replaced)
                          <strong></strong>
                      </div>
                  </span>
                  // 2
                  <em>
                      // 4
                      <div>
                          // 6 (replaced)
                          <img />
                      </div>
                  </em>
              </div>
            },
            new: html! {
              <div>
                  <span>
                      <div>
                          // Replaced
                          <em></em>
                      </div>
                  </span>
                  <em>
                      <div>
                          // Replaced
                          <i></i>
                      </div>
                  </em>
              </div>
            },
            expected: vec![
                Patch::Replace {
                    old_idx: 5,
                    new_node: &VirtualNode::element("em"),
                },
                Patch::Replace {
                    old_idx: 6,
                    new_node: &VirtualNode::element("i"),
                },
            ],
        }
        .test();
    }

    /// Verify that we use the proper new old idx when we replace a node.
    #[test]
    fn replace_node_proper_old_node_idx() {
        DiffTestCase {
            old: html! {
                <div>
                  <div><em></em></div>
                  <div></div>
                </div>
            },
            new: html! {
                <div>
                  <span></span>
                  <strong></strong>
                </div>
            },
            expected: vec![
                Patch::Replace {
                    old_idx: 1,
                    new_node: &html! { <span></span> },
                },
                Patch::Replace {
                    old_idx: 2,
                    new_node: &html! { <strong></strong> },
                },
            ],
        }
        .test();

        DiffTestCase {
            old: html! {
                <div>
                  <div>
                    <em></em>
                  </div>
                  <div></div>
                </div>
            },
            new: html! {
                <div>
                  <div>
                     <span></span>
                  </div>
                  <div></div>
                </div>
            },
            expected: vec![Patch::Replace {
                old_idx: 3,
                new_node: &html! { <span></span> },
            }],
        }
        .test();
    }

    /// Verify that we can append children to a virtual node.
    #[test]
    fn add_children() {
        DiffTestCase {
            old: html! { <div> <b></b> </div> },
            new: html! { <div> <b></b> <span></span> </div> },
            expected: vec![Patch::AppendChildren {
                parent_old_node_idx: 0,
                new_nodes: vec![&html! { <span></span> }],
            }],
        }
        .test();

        DiffTestCase {
            old: html! {
                <div>
                  <span><em></em></span>
                  <div>
                    <br />
                  </div>
                </div>
            },
            new: html! {
                <div>
                  <span><em></em></span>
                  <div>
                    <br />
                    <div><br /></div>
                    <div></div>
                  </div>
                </div>
            },
            expected: vec![Patch::AppendChildren {
                parent_old_node_idx: 2,
                new_nodes: vec![&html! { <div><br /></div> }, &html! { <div></div> }],
            }],
        }
        .test();
    }

    /// Verify that we can replace one node and add children to another node.
    #[test]
    fn replace_and_append() {
        DiffTestCase {
            old: html! {
                <div>
                  <span><em></em></span>
                  <div>
                    <br />
                  </div>
                </div>
            },
            new: html! {
                <div>
                  <i></i>
                  <div>
                    <br />
                    <div><br /></div>
                    <div></div>
                  </div>
                </div>
            },
            expected: vec![
                Patch::Replace {
                    old_idx: 1,
                    new_node: &html! { <i></i>},
                },
                Patch::AppendChildren {
                    parent_old_node_idx: 2,
                    new_nodes: vec![&html! { <div><br /></div> }, &html! { <div></div> }],
                },
            ],
        }
        .test();
    }

    /// Verify that we can truncate a node's children.
    #[test]
    fn truncate_children() {
        DiffTestCase {
            old: html! { <div> <b></b> <span></span> </div> },
            new: html! { <div> </div> },
            expected: vec![Patch::RemoveChildren {
                parent_old_node_idx: 0,
                to_remove: vec![1, 2],
            }],
        }
        .test();
        DiffTestCase {
            old: html! {
              <div>
                <span>
                  <b></b>
                  // This `i` tag will get removed
                  <i></i>
                </span>
                <div>
                  // This `em` tag will get removed
                  <em></em>
                </div>
                // This `strong` tag will get removed
                <strong></strong>
              </div>
            },
            new: html! {
              <div>
                <span>
                  <b></b>
                </span>
                <div>
                </div>
              </div>
            },
            expected: vec![
                Patch::RemoveChildren {
                    parent_old_node_idx: 0,
                    to_remove: vec![3],
                },
                Patch::RemoveChildren {
                    parent_old_node_idx: 1,
                    to_remove: vec![5],
                },
                Patch::RemoveChildren {
                    parent_old_node_idx: 2,
                    to_remove: vec![6],
                },
            ],
        }
        .test();
    }

    /// Verify that we can replace one node and truncate the children of another node.
    #[test]
    fn replace_and_truncate() {
        DiffTestCase {
            old: html! {
                <div>
                  <b>
                    <i></i>
                    <em></em>
                  </b>
                  <b></b>
                </div>
            },
            new: html! {
                <div>
                  <b>
                    <i></i>
                  </b>
                  <strong></strong>
                </div>
            },
            expected: vec![
                Patch::RemoveChildren {
                    parent_old_node_idx: 1,
                    to_remove: vec![4],
                },
                Patch::Replace {
                    old_idx: 2,
                    new_node: &html! { <strong></strong> },
                },
            ],
        }
        .test();
    }

    /// Verify that we can add an attribute to a node.
    #[test]
    fn add_attributes() {
        let mut attributes = HashMap::new();
        let id = "hello".into();
        attributes.insert("id", &id);

        DiffTestCase {
            old: html! { <div> </div> },
            new: html! { <div id="hello"> </div> },
            expected: vec![Patch::AddAttributes(0, attributes.clone())],
        }
        .test();

        DiffTestCase {
            old: html! { <div id="foobar"> </div> },
            new: html! { <div id="hello"> </div> },
            expected: vec![Patch::AddAttributes(0, attributes)],
        }
        .test();
    }

    /// Verify that we can remove an attribute from a node.
    #[test]
    fn remove_attributes() {
        DiffTestCase {
            old: html! { <div id="hey-there"></div> },
            new: html! { <div> </div> },
            expected: vec![Patch::RemoveAttributes(0, vec!["id"])],
        }
        .test();
    }

    /// Verify that we can change a node's attribute.
    #[test]
    fn change_attribute() {
        let mut attributes = HashMap::new();
        let id = "changed".into();
        attributes.insert("id", &id);

        DiffTestCase {
            old: html! { <div id="hey-there"></div> },
            new: html! { <div id="changed"> </div> },
            expected: vec![Patch::AddAttributes(0, attributes)],
        }
        .test();
    }

    /// Verify that we can change a text node's text.
    #[test]
    fn replace_text_node() {
        DiffTestCase {
            old: html! { Old },
            new: html! { New },
            expected: vec![Patch::ChangeText(0, &VText::new("New"))],
        }
        .test();
    }

    /// If an input or textarea has a value attribute we always push a patch for setting the value
    /// attribute so that we can replace anything that might have been typed into the field.
    #[test]
    fn always_pushes_patch_for_value() {
        DiffTestCase {
            old: html! { <input value="abc" /> },
            new: html! { <input value="abc" /> },
            expected: vec![Patch::ValueAttributeUnchanged(0, &"abc".into())],
        }
        .test();

        DiffTestCase {
            old: html! { <textarea value="abc" /> },
            new: html! { <textarea value="abc" /> },
            expected: vec![Patch::ValueAttributeUnchanged(0, &"abc".into())],
        }
        .test();

        DiffTestCase {
            old: html! { <textarea value="abc" /> },
            new: html! { <textarea value="def" /> },
            expected: vec![Patch::AddAttributes(
                0,
                vec![("value", &"def".into())].into_iter().collect(),
            )],
        }
        .test();
    }

    /// Verify that we push an on create elem patch if the new node has the special attribute
    /// and the old node does not.
    #[test]
    fn on_create_elem() {
        let old = VirtualNode::element("div");

        let mut new = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut new, "150");

        let mut expected = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut expected, "150");

        DiffTestCase {
            old,
            new,
            expected: vec![Patch::SpecialAttribute(
                PatchSpecialAttribute::CallOnCreateElemOnExistingNode(0, &expected),
            )],
        }
        .test();
    }

    /// Verify that if we have nested elements that need their on create element handler called we
    /// push patches for all of them.
    #[test]
    fn nested_on_create_element() {
        DiffTestCase {
            old: html! {
              <div>
                <span>
                  <div></div>
                </span>
                <br />
                <em>
                  <br />
                  <div>
                  </div>
                </em>
              </div>
            },
            new: html! {
               <div>
                <span key="150" on_create_element=|_: web_sys::Element|{} >
                  <div></div>
                </span>
                <br />
                <em key="200" on_create_element=|_: web_sys::Element|{} >
                  <br />
                  <div key="250" on_create_element=|_: web_sys::Element|{} >
                  </div>
                </em>
              </div>
            },
            expected: vec![
                Patch::SpecialAttribute(PatchSpecialAttribute::CallOnCreateElemOnExistingNode(
                    1,
                    &html! {
                    <span key="150" on_create_element=|_: web_sys::Element|{} >
                      <div></div>
                    </span>
                    },
                )),
                Patch::SpecialAttribute(PatchSpecialAttribute::CallOnCreateElemOnExistingNode(
                    3,
                    &html! {
                    <em key="200" on_create_element=|_: web_sys::Element|{} >
                      <br />
                      <div key="250" on_create_element=|_: web_sys::Element|{} >
                      </div>
                    </em>
                    },
                )),
                Patch::SpecialAttribute(PatchSpecialAttribute::CallOnCreateElemOnExistingNode(
                    6,
                    &html! {
                    <div key="250" on_create_element=|_: web_sys::Element|{} >
                    </div>
                    },
                )),
            ],
        }
        .test();
    }

    /// Verify that if two different nodes have the same on_create_elem unique identifiers we
    /// do not push a CallOnCreateElem patch.
    #[test]
    fn same_on_create_elem_id() {
        let mut old = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut old, "70");

        let mut new = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut new, "70");

        DiffTestCase {
            old,
            new,
            expected: vec![],
        }
        .test();
    }

    /// Verify that if two different nodes have different on_create_elem unique identifiers we push
    /// a patch to call the new on_create_elem.
    #[test]
    fn different_on_create_elem_id() {
        let mut old = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut old, "50");

        let mut new = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut new, "99");

        let mut expected = VirtualNode::element("div");
        set_on_create_elem_with_unique_id(&mut expected, "99");

        DiffTestCase {
            old,
            new,
            expected: vec![Patch::SpecialAttribute(
                PatchSpecialAttribute::CallOnCreateElemOnExistingNode(0, &expected),
            )],
        }
        .test();
    }

    /// Verify that we push an on remove elem patch if the new node has the special attribute
    /// and the old node does not.
    #[test]
    fn on_remove_elem_for_replaced_elem() {
        let mut old = VirtualNode::element("div");
        set_on_remove_elem_with_unique_id(&mut old, "150");

        let expected = {
            let mut old = VirtualNode::element("div");
            set_on_remove_elem_with_unique_id(&mut old, "150");

            old
        };

        let new = VirtualNode::element("span");

        DiffTestCase {
            old,
            new,
            expected: vec![
                Patch::SpecialAttribute(PatchSpecialAttribute::CallOnRemoveElem(0, &expected)),
                Patch::Replace {
                    old_idx: 0,
                    new_node: &VirtualNode::element("span"),
                },
            ],
        }
        .test();
    }

    /// Verify that we push on remove element patches for replaced children, their replaced
    /// children, etc.
    #[test]
    fn on_remove_elem_for_replaced_children_recursively() {
        DiffTestCase {
            old: html! {
              <div>
                <em key="key" on_remove_element=||{}>
                  <br />
                  <strong key="key" on_remove_element=||{}></strong>
                </em>
                <br />
                <div key="key" on_remove_element=||{}></div>
              </div>
            },
            new: html! {
              <span></span>
            },
            expected: vec![
                Patch::Replace {
                    old_idx: 0,
                    new_node: &VirtualNode::element("span"),
                },
                Patch::SpecialAttribute(PatchSpecialAttribute::CallOnRemoveElem(
                    1,
                    &html! {
                    <em key="key" on_remove_element=||{}>
                      <br />
                      <strong key="key" on_remove_element=||{}></strong>
                    </em>
                    },
                )),
                Patch::SpecialAttribute(PatchSpecialAttribute::CallOnRemoveElem(
                    3,
                    &html! {
                    <div key="key" on_remove_element=||{}></div>
                    },
                )),
                Patch::SpecialAttribute(PatchSpecialAttribute::CallOnRemoveElem(
                    5,
                    &html! {
                    <strong key="key" on_remove_element=||{}></strong>
                    },
                )),
            ],
        }
        .test();
    }

    /// Verify that we push on remove element patches for truncated children.
    #[test]
    fn on_remove_elem_for_truncated_children_recursively() {
        let mut grandchild = VirtualNode::element("strong");
        set_on_remove_elem_with_unique_id(&mut grandchild, "key");

        let mut child = VirtualNode::element("em");
        set_on_remove_elem_with_unique_id(&mut child, "key");

        child.as_velement_mut().unwrap().children.push(grandchild);

        let old = html! {
            <div>
                <span>
                  <em></em>
                </span>

                // Gets truncated.
                {child}
            </div>
        };

        let new = html! {
            <div>
                <span>
                  <em></em>
                </span>
            </div>
        };

        let expected_child = {
            let mut grandchild = VirtualNode::element("strong");
            set_on_remove_elem_with_unique_id(&mut grandchild, "key");

            let mut child = VirtualNode::element("em");
            set_on_remove_elem_with_unique_id(&mut child, "key");
            child.as_velement_mut().unwrap().children.push(grandchild);

            child
        };
        let expected_grandchild = {
            let mut grandchild = VirtualNode::element("strong");
            set_on_remove_elem_with_unique_id(&mut grandchild, "key");
            grandchild
        };

        DiffTestCase {
            old,
            new,
            expected: vec![
                Patch::RemoveChildren {
                    parent_old_node_idx: 0,
                    to_remove: vec![2],
                },
                Patch::SpecialAttribute(PatchSpecialAttribute::CallOnRemoveElem(
                    2,
                    &expected_child,
                )),
                Patch::SpecialAttribute(PatchSpecialAttribute::CallOnRemoveElem(
                    4,
                    &expected_grandchild,
                )),
            ],
        }
        .test();
    }

    /// Verify that when patching attributes, if the old has an on remove element callback but the
    /// new node does not, we call the on remove element callback.
    ///
    /// But only for that element, since the element's below it might not get removed from the
    /// DOM.
    #[test]
    fn new_node_does_not_have_on_remove_elem() {
        let old_child = on_remove_node_with_on_remove_child();
        let mut old = html! {
            <div>
                {old_child}
            </div>
        };
        set_on_remove_elem_with_unique_id(&mut old, "some-key");

        let expected = {
            let old_child = on_remove_node_with_on_remove_child();
            let mut old = html! {
                <div>
                    {old_child}
                </div>
            };
            set_on_remove_elem_with_unique_id(&mut old, "some-key");

            old
        };

        let new_child = on_remove_node_with_on_remove_child();
        let new = html! {
            <div>
                {new_child}
            </div>
        };

        DiffTestCase {
            old,
            new,
            expected: vec![Patch::SpecialAttribute(
                PatchSpecialAttribute::CallOnRemoveElem(0, &expected),
            )],
        }
        .test();
    }

    /// Verify that when patching attributes, if the old and new node are of the same tag type but
    /// have different on remove element ID, a patch is pushed.
    ///
    /// But only for that element, since the element's below it might not get removed from the
    /// DOM.
    #[test]
    fn different_on_remove_elem_id() {
        let old_child = on_remove_node_with_on_remove_child();
        let mut old = html! {
            <div>
                {old_child}
            </div>
        };
        set_on_remove_elem_with_unique_id(&mut old, "start");

        let expected = {
            let old_child = on_remove_node_with_on_remove_child();
            let mut old = html! {
                <div>
                    {old_child}
                </div>
            };
            set_on_remove_elem_with_unique_id(&mut old, "start");

            old
        };

        let new_child = on_remove_node_with_on_remove_child();
        let mut new = html! {
            <div>
                {new_child}
            </div>
        };
        set_on_remove_elem_with_unique_id(&mut new, "end");

        DiffTestCase {
            old,
            new,
            expected: vec![Patch::SpecialAttribute(
                PatchSpecialAttribute::CallOnRemoveElem(0, &expected),
            )],
        }
        .test();
    }

    /// Verify that if the old and new node have the same on remove element ID, no patch is pushed.
    #[test]
    fn same_on_remove_elem_id() {
        let mut old = VirtualNode::element("div");
        set_on_remove_elem_with_unique_id(&mut old, "same");

        let mut new = VirtualNode::element("div");
        set_on_remove_elem_with_unique_id(&mut new, "same");

        DiffTestCase {
            old,
            new,
            expected: vec![],
        }
        .test();
    }

    /// Verify that if the old node and new node have the same dangerous_inner_html we do not push
    /// an SetDangerousInnerHtml patch.
    #[test]
    fn same_dangerous_inner_html() {
        let mut old = VirtualNode::element("div");
        set_dangerous_inner_html(&mut old, "hi");

        let mut new = VirtualNode::element("div");
        set_dangerous_inner_html(&mut new, "hi");

        DiffTestCase {
            old,
            new,
            expected: vec![],
        }
        .test();
    }

    /// Verify that if the new node has dangerous_inner_html that is different from the old node's,
    /// we push a patch to set the new inner html.
    #[test]
    fn different_dangerous_inner_html() {
        let mut old = VirtualNode::element("div");
        set_dangerous_inner_html(&mut old, "old");

        let mut new = VirtualNode::element("div");
        set_dangerous_inner_html(&mut new, "new");

        let mut expected = VirtualNode::element("div");
        set_dangerous_inner_html(&mut expected, "new");

        DiffTestCase {
            old,
            new,
            expected: vec![Patch::SpecialAttribute(
                PatchSpecialAttribute::SetDangerousInnerHtml(0, &expected),
            )],
        }
        .test();
    }

    /// Verify that if the new node does not have dangerous_inner_html and the old node does, we
    /// push a patch to truncate all children along with a patch to push the new node's
    /// children.
    #[test]
    fn remove_dangerous_inner_html() {
        let mut old = VirtualNode::element("div");
        set_dangerous_inner_html(&mut old, "hi");

        let new = html! { <div><em></em></div> };

        DiffTestCase {
            old,
            new,
            expected: vec![
                Patch::SpecialAttribute(PatchSpecialAttribute::RemoveDangerousInnerHtml(0)),
                Patch::AppendChildren {
                    parent_old_node_idx: 0,
                    new_nodes: vec![&VirtualNode::element("em")],
                },
            ],
        }
        .test();
    }

    /// Verify that if a node already had a event and we are patching it with another
    /// event we do not create a patch for setting the events ID.
    #[test]
    fn does_not_set_events_id_if_already_had_events() {
        let mut old = VElement::new("div");
        old.events.insert(onclick_name(), mock_event_handler());

        let mut new = VElement::new("div");
        new.events.insert(onclick_name(), mock_event_handler());

        DiffTestCase {
            old: VirtualNode::Element(old),
            new: VirtualNode::Element(new),
            expected: vec![],
        }
        .test();
    }

    /// Verify that if 5 earlier node were replaced replaced by 5 different nodes, we do not
    /// reset the events ID for nodes that come after it since the total number of nodes has not
    /// changed.
    ///
    /// This test should also cover cases where the same number of earlier nodes are
    /// truncated / appended, since our implementation just checks whether or not the new node IDX
    /// is equal to the old node IDX.
    /// If not, then that node and every node after it needs its events ID reset
    /// (if they have events).
    #[test]
    fn does_not_reset_events_id_if_earlier_node_replaced_by_same_number_of_nodes() {
        let old = html! {
            <div>
                // This node gets replaced, but with the same number of nodes.
                <span>
                    <em>
                        <area />
                    </em>
                </span>

                <strong onclick=|| {}>
                    <div></div>
                    <a onclick=|| {}></a>
                </strong>
            </div>
        };

        let new = html! {
            <div>
                <div>
                    <ul>
                        <li> </li>
                    </ul>
                </div>

                <strong onclick=|| {}>
                    <div></div>
                    <a onclick=|| {}></a>
                </strong>
            </div>
        };

        DiffTestCase {
            old,
            new,
            expected: vec![Patch::Replace {
                old_idx: 1,
                new_node: &html! {<div> <ul> <li> </li> </ul> </div>},
            }],
        }
        .test();
    }

    /// Verify that if we previously had events but we no longer have any events we push a patch
    /// to remove the virtual events.
    #[test]
    fn removes_events_if_no_more_events() {
        let mut old = VElement::new("div");
        old.events.insert(onclick_name(), mock_event_handler());

        let new = VElement::new("div");

        DiffTestCase {
            old: VirtualNode::Element(old),
            new: VirtualNode::Element(new),
            expected: vec![Patch::RemoveEvents(
                0,
                vec![(&EventName::ONCLICK, &mock_event_handler())]
                    .into_iter()
                    .collect(),
            )],
        }
        .test();
    }

    /// Verify that if an element has added and removed multiple non-delegated events, the remove
    /// event listener patches come before the add event listener patches.
    /// This ensures that we can look up the old functions in the `EventsByNodeIdx` that we'll need
    /// to pass into .remove_event_listener() before the SetEventListeners patch overwrites those
    /// functions.
    #[test]
    fn remove_event_patches_come_before_add_event_patches() {
        let mut old = VElement::new("div");
        old.events.insert(oninput_name(), mock_event_handler());

        let mut new = VElement::new("div");
        new.events.insert(onmousemove_name(), mock_event_handler());

        DiffTestCase {
            old: VirtualNode::Element(old),
            new: VirtualNode::Element(new),
            expected: vec![
                Patch::RemoveEvents(0, vec![(&oninput_name(), &mock_event_handler())]),
                Patch::AddEvents(
                    0,
                    vec![(&onmousemove_name(), &mock_event_handler())]
                        .into_iter()
                        .collect(),
                ),
            ],
        }
        .test();
    }

    /// Verify that if a node has events but the node is replaced we push a patch to remove all
    /// of its events from the EventsByNodeIdx.
    /// We ensure that this event removal patch should come before the patch to replace the node,
    /// so that we don't accidentally remove events that were for the node that replaced it.
    #[test]
    fn remove_all_tracked_events_if_replaced() {
        let mut old = VElement::new("div");
        old.events.insert(oninput_name(), mock_event_handler());

        let new = VElement::new("some-other-element");

        DiffTestCase {
            old: VirtualNode::Element(old),
            new: VirtualNode::Element(new),
            expected: vec![
                Patch::RemoveAllVirtualEventsWithNodeIdx(0),
                Patch::Replace {
                    old_idx: 0,
                    new_node: &VirtualNode::Element(VElement::new("some-other-element")),
                },
            ],
        }
        .test();
    }

    /// Verify that if a node's ancestor (parent, grandparent, ..etc) was replaced we push a patch
    /// to remove all of its events from the EventsByNodeIdx.
    /// We ensure that this event removal patch should come before the patch to replace the node,
    /// so that we don't accidentally remove events that were for the node that replaced it.
    #[test]
    fn removes_tracked_events_if_ancestor_replaced() {
        // node idx 0
        let mut old = VElement::new("div");
        // node idx 1
        old.children.push(VirtualNode::Element(VElement::new("a")));
        // node idx 2
        old.children.push(VirtualNode::text("b"));

        // node idx 3
        let mut child_of_old = VElement::new("div");
        child_of_old
            .events
            .insert(oninput_name(), mock_event_handler());
        old.children.push(VirtualNode::Element(child_of_old));

        let new = VElement::new("some-other-element");

        DiffTestCase {
            old: VirtualNode::Element(old),
            new: VirtualNode::Element(new),
            expected: vec![
                Patch::Replace {
                    old_idx: 0,
                    new_node: &VirtualNode::Element(VElement::new("some-other-element")),
                },
                Patch::RemoveAllVirtualEventsWithNodeIdx(3),
            ],
        }
        .test();
    }

    /// Verify that if a child node is truncated and it had events we push a patch to remove all
    /// of its events from the EventsByNodeIdx
    #[test]
    fn remove_tracked_events_if_truncated() {
        let mut old = VElement::new("div");
        let mut child_of_old = VElement::new("div");
        child_of_old
            .events
            .insert(oninput_name(), mock_event_handler());
        old.children.push(VirtualNode::Element(child_of_old));

        let new = VElement::new("div");

        DiffTestCase {
            old: VirtualNode::Element(old),
            new: VirtualNode::Element(new),
            expected: vec![
                Patch::RemoveChildren {
                    parent_old_node_idx: 0,
                    to_remove: vec![1],
                },
                Patch::RemoveAllVirtualEventsWithNodeIdx(1),
            ],
        }
        .test();
    }

    /// Verify that we can prepend an element to a keyed list.
    #[test]
    fn prepend_new_element_to_keyed_list() {
        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
              </div>
            },
            new: html! {
              <div>
                <span></span>
                <em key="a"></em>
              </div>
            },
            expected: vec![Patch::InsertBefore {
                anchor_old_node_idx: 1,
                new_nodes: vec![&VirtualNode::element("span")],
            }],
        }
        .test();

        DiffTestCase {
            old: html! {
              <div>
                // At the time of writing this test adding these extra nodes made the test
                // fail since we had a bug in our old node index calculation log.
                <div>
                  <div></div>
                </div>

                <div>
                  <em key="a"></em>
                </div>
              </div>
            },
            new: html! {
              <div>
                <div>
                  <div></div>
                </div>

                <div>
                  <span key="new"></span>
                  <em key="a"></em>
                </div>
              </div>
            },
            expected: vec![Patch::InsertBefore {
                anchor_old_node_idx: 4,
                new_nodes: vec![&node_with_key("span", "new")],
            }],
        }
        .test();

        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
              </div>
            },
            new: html! {
              <div>
                <footer></footer>
                <span></span>
                <em key="a"></em>
              </div>
            },
            expected: vec![Patch::InsertBefore {
                anchor_old_node_idx: 1,
                new_nodes: vec![
                    &VirtualNode::element("footer"),
                    &VirtualNode::element("span"),
                ],
            }],
        }
        .test();
    }

    /// Verify that if we move a node to displace one of its previous siblings and we modify
    /// one of the moved node's children this modification has the correct node index.
    #[test]
    fn move_keyed_node_before_and_append_children_to_that_nodes_child() {
        DiffTestCase {
            old: html! {
              <div>
                <span key="a">
                  <img />
                </span>
                <em key="b">
                  <div>
                  </div>
                </em>
              </div>
            },
            new: html! {
              <div>
                <em key="b">
                  <div>
                    <br />
                  </div>
                </em>
                <span key="a">
                  <img />
                </span>
              </div>
            },
            expected: vec![
                Patch::MoveNodesBefore {
                    anchor_old_node_idx: 1,
                    to_move: vec![2],
                },
                Patch::AppendChildren {
                    parent_old_node_idx: 4,
                    new_nodes: vec![&VirtualNode::element("br")],
                },
            ],
        }
        .test();
    }

    /// Verify that we can remove an unkeyed nodes that appears in between two keyed nodes.
    #[test]
    fn remove_unkeyed_node_in_between_keyed_nodes() {
        DiffTestCase {
            old: html! {
              <div>
                <div key="a"></div>
                <span></span>
                <strong></strong>
                <em key="b"></em>
              </div>
            },
            new: html! {
              <div>
                <div key="a"></div>
                <em key="b"></em>
              </div>
            },
            expected: vec![Patch::RemoveChildren {
                parent_old_node_idx: 0,
                to_remove: vec![2, 3],
            }],
        }
        .test();

        DiffTestCase {
            old: html! {
              <div>
                <div key="a"></div>
                <span>
                  // Make sure that on remove element gets called for the children of a
                  // removed unkeyed node.
                  <strong key="123" on_remove_element=||{}></strong>
                </span>
                <em key="b"></em>
              </div>
            },
            new: html! {
              <div>
                <div key="a"></div>
                <em key="b"></em>
              </div>
            },
            expected: vec![
                Patch::RemoveChildren {
                    parent_old_node_idx: 0,
                    to_remove: vec![2],
                },
                Patch::SpecialAttribute(PatchSpecialAttribute::CallOnRemoveElem(
                    4,
                    &node_with_on_remove_element("strong", "123"),
                )),
            ],
        }
        .test();
    }

    /// Verify that we can append an element to a keyed list.
    #[test]
    fn append_new_element_to_keyed_list() {
        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
              </div>
            },
            new: html! {
              <div>
                <em key="a"></em>
                <span></span>
              </div>
            },
            expected: vec![Patch::AppendChildren {
                parent_old_node_idx: 0,
                new_nodes: vec![&VirtualNode::element("span")],
            }],
        }
        .test();

        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
              </div>
            },
            new: html! {
              <div>
                <em key="a"></em>
                <span></span>
                <strong></strong>
              </div>
            },
            expected: vec![Patch::AppendChildren {
                parent_old_node_idx: 0,
                new_nodes: vec![
                    &VirtualNode::element("span"),
                    &VirtualNode::element("strong"),
                ],
            }],
        }
        .test();
    }

    /// Verify that we can assert a new element into the middle of a keyed list.
    #[test]
    fn insert_new_element_into_middle_of_keyed_list() {
        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
                <footer key="b"></footer>
              </div>
            },
            new: html! {
              <div>
                <em key="a"></em>
                // Insert non-keyed element
                <span></span>
                <footer key="b"></footer>
              </div>
            },
            expected: vec![Patch::InsertBefore {
                anchor_old_node_idx: 2,
                new_nodes: vec![&VirtualNode::element("span")],
            }],
        }
        .test();

        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
                <footer key="b"></footer>
              </div>
            },
            new: html! {
              <div>
                <em key="a"></em>
                // Insert keyed element
                <span key="new-key"></span>
                <footer key="b"></footer>
              </div>
            },
            expected: vec![Patch::InsertBefore {
                anchor_old_node_idx: 2,
                new_nodes: vec![&node_with_key("span", "new-key")],
            }],
        }
        .test();

        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
                <footer key="b"></footer>
              </div>
            },
            new: html! {
              <div>
                <em key="a"></em>
                <span></span>
                <div></div>
                <footer key="b"></footer>
              </div>
            },
            expected: vec![Patch::InsertBefore {
                anchor_old_node_idx: 2,
                new_nodes: vec![&VirtualNode::element("span"), &VirtualNode::element("div")],
            }],
        }
        .test();
    }

    /// Verify that we can re-order a list of keyed elements.
    #[test]
    fn reorder_keyed_elements() {
        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
                <span key="b"></span>
              </div>
            },
            new: html! {
              <div>
                <span key="b"></span>
                <em key="a"></em>
              </div>
            },
            expected: vec![Patch::MoveNodesBefore {
                anchor_old_node_idx: 1,
                to_move: vec![2],
            }],
        }
        .test();

        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
                <span key="b"></span>
                <span key="c"></span>
                <span key="d"></span>
                <span key="e"></span>
              </div>
            },
            new: html! {
              <div>
                <span key="e"></span>
                <em key="a"></em>
                <span key="b"></span>
                <span key="c"></span>
                <span key="d"></span>
              </div>
            },
            expected: vec![Patch::MoveNodesBefore {
                anchor_old_node_idx: 1,
                to_move: vec![5],
            }],
        }
        .test();

        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
                <span key="BBB"></span>
                <span key="c"></span>
                <span key="DDD"></span>
                <span key="e"></span>
              </div>
            },
            new: html! {
              <div>
                <em key="a"></em>
                <span key="DDD"></span>
                <span key="c"></span>
                <span key="BBB"></span>
                <span key="e"></span>
              </div>
            },
            expected: vec![Patch::MoveNodesBefore {
                anchor_old_node_idx: 2,
                to_move: vec![4, 3],
            }],
        }
        .test();

        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
                <footer key="b"></footer>
              </div>
            },
            new: html! {
              <div>
                <footer key="b"></footer>
                <span></span>
                <div></div>
                <em key="a"></em>
              </div>
            },
            expected: vec![
                Patch::MoveNodesBefore {
                    anchor_old_node_idx: 1,
                    to_move: vec![2],
                },
                Patch::InsertBefore {
                    anchor_old_node_idx: 1,
                    new_nodes: vec![&VirtualNode::element("span"), &VirtualNode::element("div")],
                },
            ],
        }
        .test();
    }

    /// Verify that we can re-order a list of keyed elements.
    #[test]
    fn swap_keyed_and_unkeyed_elements() {
        DiffTestCase {
            old: html! {
              <div>
                <em></em>
                <span key="b"></span>
              </div>
            },
            new: html! {
              <div>
                <span key="b"></span>
                <em></em>
              </div>
            },
            expected: vec![Patch::MoveToEndOfSiblings {
                parent_old_node_idx: 0,
                siblings_to_move: vec![1],
            }],
        }
        .test();

        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
                <span></span>
              </div>
            },
            new: html! {
              <div>
                <span></span>
                <em key="a"></em>
              </div>
            },
            expected: vec![Patch::MoveNodesBefore {
                anchor_old_node_idx: 1,
                to_move: vec![2],
            }],
        }
        .test();

        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
                <span class="hello"></span>
              </div>
            },
            new: html! {
              <div>
                <span></span>
                <em key="a"></em>
              </div>
            },
            expected: vec![
                Patch::MoveNodesBefore {
                    anchor_old_node_idx: 1,
                    to_move: vec![2],
                },
                Patch::RemoveAttributes(2, vec!["class"]),
            ],
        }
        .test();
    }

    /// Verify that we can move a node B before another node A and also truncate node B's
    /// children.
    #[test]
    fn move_before_and_truncate_same_node() {
        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
                <span key="b">
                  <img />
                </span>
              </div>
            },
            new: html! {
              <div>
                <span key="b">
                </span>
                <em key="a"></em>
              </div>
            },
            expected: vec![
                Patch::MoveNodesBefore {
                    anchor_old_node_idx: 1,
                    to_move: vec![2],
                },
                Patch::RemoveChildren {
                    parent_old_node_idx: 2,
                    to_remove: vec![3],
                },
            ],
        }
        .test();
    }

    /// Verify that we generate the correct patches for both moving and inserting some nodes before
    /// another node.
    #[test]
    fn move_and_insert_before() {
        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
                <footer key="b"></footer>
                <span key="c"></span>
                <span key="d"></span>
                <span key="e"></span>
              </div>
            },
            new: html! {
              <div>
                <em key="a"></em>
                <span key="c"></span>
                <header></header>
                <span key="d"></span>
                <strong></strong>
                <span key="e"></span>
                <header></header>
                <img />
                <footer key="b"></footer>
              </div>
            },
            expected: vec![
                Patch::InsertBefore {
                    anchor_old_node_idx: 4,
                    new_nodes: vec![&VirtualNode::element("header")],
                },
                Patch::InsertBefore {
                    anchor_old_node_idx: 5,
                    new_nodes: vec![&VirtualNode::element("strong")],
                },
                Patch::AppendChildren {
                    parent_old_node_idx: 0,
                    new_nodes: vec![
                        &VirtualNode::element("header"),
                        &VirtualNode::element("img"),
                    ],
                },
                Patch::MoveToEndOfSiblings {
                    parent_old_node_idx: 0,
                    siblings_to_move: vec![2],
                },
            ],
        }
        .test();
    }

    /// Verify that if we have siblings "A B C" we
    /// - generate two move patches to get to "C B A".
    ///   That is, C gets moved before A, then B gets moved before A.
    /// - generate two move patches to get to "B C A".
    ///   That is, B gets moved before A, then C gets moved before A.
    #[test]
    fn move_two_keyed_elements_before_another() {
        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
                <span key="b"></span>
                <span key="c"></span>
              </div>
            },
            new: html! {
              <div>
                <span key="c"></span>
                <span key="b"></span>
                <em key="a"></em>
              </div>
            },
            expected: vec![Patch::MoveNodesBefore {
                anchor_old_node_idx: 1,
                to_move: vec![3, 2],
            }],
        }
        .test();

        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
                <span key="b"></span>
                <span key="c"></span>
              </div>
            },
            new: html! {
              <div>
                <span key="b"></span>
                <span key="c"></span>
                <em key="a"></em>
              </div>
            },
            expected: vec![Patch::MoveToEndOfSiblings {
                parent_old_node_idx: 0,
                siblings_to_move: vec![1],
            }],
        }
        .test();
    }

    /// Verify that if two elements have the same tag and key and are in the same position then
    /// they don't get swapped
    #[test]
    fn same_tag_same_key() {
        DiffTestCase {
            old: html! {
              <div>
                <span key="a"></span>
              </div>
            },
            new: html! {
              <div>
                <span key="a"></span>
              </div>
            },
            expected: vec![],
        }
        .test();

        DiffTestCase {
            old: html! {
              <div>
                <span key="a"></span>
                <em key="b"></em>
              </div>
            },
            new: html! {
              <div>
                <span key="a"></span>
                <em key="b"></em>
              </div>
            },
            expected: vec![],
        }
        .test();
    }

    /// Verify that if two elements have the same tag but different keys we do not replace the
    /// element.
    #[test]
    fn same_tag_different_key() {
        DiffTestCase {
            old: html! {
              <div>
                <span key="a"></span>
              </div>
            },
            new: html! {
              <div>
                <span key="b"></span>
              </div>
            },
            expected: vec![],
        }
        .test();

        DiffTestCase {
            old: html! {
              <div>
                <span key="a" id="111"></span>
              </div>
            },
            new: html! {
              <div>
                <span key="b" id="222"></span>
              </div>
            },
            expected: vec![Patch::AddAttributes(
                1,
                vec![("id", &"222".into())].into_iter().collect(),
            )],
        }
        .test();
    }

    /// Verify that if two elements have the same key but different tags we replace the element.
    #[test]
    fn same_key_different_tag() {
        let mut new = VirtualNode::element("span");
        new.as_velement_mut()
            .unwrap()
            .attrs
            .insert("key".to_string(), AttributeValue::String("a".to_string()));

        DiffTestCase {
            old: html! {
              <div>
                <em key="a"></em>
              </div>
            },
            new: html! {
              <div>
                <span key="a"></span>
              </div>
            },
            expected: vec![Patch::Replace {
                old_idx: 1,
                new_node: &new,
            }],
        }
        .test();
    }

    /// Verify that when adding or removing nodes from a list of children we try to avoid replacing
    /// focusable elements such as inputs and textarea elements and that instead, when possible, we
    /// simply move them.
    /// We want to move these elements instead of recreating them so that if one of them is focused
    /// it doesn't lose that focus.
    #[test]
    fn preserve_focusable_elements() {
        // Prepend before one input.
        DiffTestCase {
            old: html! {
                <div>
                  <input />
                </div>
            },
            new: html! {
                <div>
                  <br />
                  <input />
                </div>
            },
            expected: vec![Patch::InsertBefore {
                anchor_old_node_idx: 1,
                new_nodes: vec![&VirtualNode::element("br")],
            }],
        }
        .test();

        // Prepend before one textarea.
        DiffTestCase {
            old: html! {
                <div>
                  <textarea />
                </div>
            },
            new: html! {
                <div>
                  <br />
                  <textarea />
                </div>
            },
            expected: vec![Patch::InsertBefore {
                anchor_old_node_idx: 1,
                new_nodes: vec![&VirtualNode::element("br")],
            }],
        }
        .test();

        // Swap two focusable elements with different tags.
        DiffTestCase {
            old: html! {
                <div>
                  <input />
                  <textarea />
                </div>
            },
            new: html! {
                <div>
                  <textarea />
                  <input />
                </div>
            },
            expected: vec![Patch::MoveNodesBefore {
                anchor_old_node_idx: 1,
                to_move: vec![2],
            }],
        }
        .test();
    }

    /// Verify that if a focusable element has an explicit key the explicit key takes priority.
    ///
    /// We confirm this by giving an explicit key to a focusable element, moving it, and ensuring
    /// that the keyed element gets moved.
    ///
    /// If explicit keys did not take precedence over the implicit focusable element key then we
    /// would not have moved the input element since we would have thought that nothing had changed.
    #[test]
    fn prioritizes_explicit_key_over_focusable_element_key() {
        DiffTestCase {
            old: html! {
                <div>
                  <input key="keyed" />
                  <input />
                </div>
            },
            new: html! {
                <div>
                  <input />
                  <input key="keyed" />
                </div>
            },
            expected: vec![Patch::MoveNodesBefore {
                anchor_old_node_idx: 1,
                to_move: vec![2],
            }],
        }
        .test();
    }

    fn node_with_key(tag: &'static str, key: &'static str) -> VirtualNode {
        let mut node = VirtualNode::element(tag);
        node.as_velement_mut()
            .unwrap()
            .attrs
            .insert("key".to_string(), key.into());

        node
    }

    fn node_with_on_remove_element(tag: &'static str, key: &'static str) -> VirtualNode {
        let mut node = node_with_key(tag, key);
        set_on_remove_elem_with_unique_id(&mut node, key);

        node
    }

    fn set_on_create_elem_with_unique_id(node: &mut VirtualNode, on_create_elem_id: &'static str) {
        node.as_velement_mut()
            .unwrap()
            .special_attributes
            .set_on_create_element(on_create_elem_id, |_: web_sys::Element| {});
    }

    fn set_on_remove_elem_with_unique_id(node: &mut VirtualNode, on_remove_elem_id: &'static str) {
        node.as_velement_mut()
            .unwrap()
            .special_attributes
            .set_on_remove_element(on_remove_elem_id, |_: web_sys::Element| {});
    }

    fn set_dangerous_inner_html(node: &mut VirtualNode, html: &str) {
        node.as_velement_mut()
            .unwrap()
            .special_attributes
            .dangerous_inner_html = Some(html.to_string());
    }

    /// Return a node that has an on remove element function.
    ///
    /// This node has a child that also has an on remove element function.
    ///
    /// <div>
    ///   <div></div>
    /// </div>
    fn on_remove_node_with_on_remove_child() -> VirtualNode {
        let mut child = VirtualNode::element("div");
        set_on_remove_elem_with_unique_id(&mut child, "555");

        let mut node = VirtualNode::element("div");
        set_on_remove_elem_with_unique_id(&mut node, "666");

        node.as_velement_mut().unwrap().children.push(child);

        node
    }

    fn mock_event_handler() -> EventHandler {
        EventHandler::UnsupportedSignature(EventAttribFn(Rc::new(Box::new(JsValue::NULL))))
    }

    fn onclick_name() -> EventName {
        "onclick".into()
    }

    fn oninput_name() -> EventName {
        "oninput".into()
    }

    fn onmousemove_name() -> EventName {
        "onmousemove".into()
    }
}
