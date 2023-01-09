//! Tests diffing and patching lists where the siblings have `key` properties set.
//!
//! These tests do not assert on the patches that are generated, only that the final patched element
//! is correct.
//!
//! The tests in `crates/percy-dom/src/diff.rs` assert on the exact patches generated for keyed
//! lists.
//!
//! To run all tests in this file:
//!
//! wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists

extern crate wasm_bindgen_test;
extern crate web_sys;

use crate::diff_patch_test_case::DiffPatchTest;
use percy_dom::Patch;
use std::collections::HashSet;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::Node;

use crate::testing_utilities::create_node_and_events_and_append_to_document;
use percy_dom::prelude::*;
use virtual_node::event::{VirtualEvents, ELEMENT_EVENTS_ID_PROP};

wasm_bindgen_test_configure!(run_in_browser);

mod diff_patch_test_case;
mod testing_utilities;

/// Verify that the insert before patch inserts nodes before another node.
///
/// _InsertBefore is one of the patches used for some keyed lists._
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- insert_before_patch_inserts_nodes
#[wasm_bindgen_test]
fn insert_before_patch_inserts_nodes() {
    let vnode = html! {
      <div>
        <br />
        <span></span>
      </div>
    };

    let (node, mut events) = create_node_and_events_and_append_to_document(vnode);

    percy_dom::patch(
        node.clone(),
        &VirtualNode::element("..."),
        &mut events,
        &vec![Patch::InsertBefore {
            anchor_old_node_idx: 2,
            new_nodes: vec![
                &html! { <em> </em> },
                &html! { <div> </div> },
                &html! { <img /> },
            ],
        }],
    )
    .unwrap();

    let expected_child_tags = vec!["br", "em", "div", "img", "span"];
    assert_dom_children_and_events_children_match(&node, &events, &expected_child_tags);
}

/// Verify that we can have multiple insert before patches targeting the same node.
///
/// This test helps check that our patching algorithm will continue to insert before the same node
/// even as if its index amongst its siblings changes between patches.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- multiple_insert_before_patches
#[wasm_bindgen_test]
fn multiple_insert_before_patches() {
    let vnode = html! {
      <div>
        <img />
        <br />
      </div>
    };

    let (node, mut events) = create_node_and_events_and_append_to_document(vnode);

    percy_dom::patch(
        node.clone(),
        &VirtualNode::element("..."),
        &mut events,
        &vec![
            Patch::InsertBefore {
                anchor_old_node_idx: 2,
                new_nodes: vec![&html! { <em> </em> }],
            },
            Patch::InsertBefore {
                anchor_old_node_idx: 2,
                new_nodes: vec![&html! { <span> </span> }],
            },
        ],
    )
    .unwrap();

    let expected_child_tags = vec!["img", "em", "span", "br"];
    assert_dom_children_and_events_children_match(&node, &events, &expected_child_tags);
}

/// Verify that the move nodes before patch moves nodes before another node.
///
/// _MoveNodesBefore is one of the patches used for some keyed lists._
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- move_nodes_before_patch_moves_nodes
#[wasm_bindgen_test]
fn move_nodes_before_patch_moves_nodes() {
    let vnode = html! {
      <div>
        <span></span>
        <em></em>
        <img />
        <strong></strong>
        <br />
      </div>
    };

    let (node, mut events) = create_node_and_events_and_append_to_document(vnode);

    percy_dom::patch(
        node.clone(),
        &VirtualNode::element("..."),
        &mut events,
        &vec![Patch::MoveNodesBefore {
            anchor_old_node_idx: 2,
            to_move: vec![3, 5],
        }],
    )
    .unwrap();

    let expected_child_tags = vec!["span", "img", "br", "em", "strong"];
    assert_dom_children_and_events_children_match(&node, &events, &expected_child_tags);
}

/// Verify that we can have multiple move nodes before patches targeting the same node.
///
/// This test helps check that our patching algorithm will continue to move the correct node
/// even as if its index amongst its siblings changes between patches.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- multiple_move_nodes_before_patches
#[wasm_bindgen_test]
fn multiple_move_nodes_before_patches() {
    let vnode = html! {
      <div>
        <span></span>
        <img />
        <strong></strong>
        <br />
      </div>
    };

    let (node, mut events) = create_node_and_events_and_append_to_document(vnode);

    percy_dom::patch(
        node.clone(),
        &VirtualNode::element("..."),
        &mut events,
        &vec![
            Patch::MoveNodesBefore {
                anchor_old_node_idx: 2,
                to_move: vec![3],
            },
            Patch::MoveNodesBefore {
                anchor_old_node_idx: 2,
                to_move: vec![4],
            },
        ],
    )
    .unwrap();

    let expected_child_tags = vec!["span", "strong", "br", "img"];
    assert_dom_children_and_events_children_match(&node, &events, &expected_child_tags);
}

/// Verify that the move nodes to end of siblings patch appends nodes to the parent.
///
/// _MoveToEndOfSiblings is one of the patches that gets used for some keyed lists._
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- move_to_end_of_siblings_patch_moves_nodes
#[wasm_bindgen_test]
fn move_to_end_of_siblings_patch_moves_nodes() {
    let vnode = html! {
      <div>
        <span></span>
        <em></em>
        <img />
        <strong></strong>
        <br />
      </div>
    };

    let (node, mut events) = create_node_and_events_and_append_to_document(vnode);

    percy_dom::patch(
        node.clone(),
        &VirtualNode::element("..."),
        &mut events,
        &vec![Patch::MoveToEndOfSiblings {
            parent_old_node_idx: 0,
            siblings_to_move: vec![2, 4],
        }],
    )
    .unwrap();

    let expected_child_tags = vec!["span", "img", "br", "em", "strong"];
    assert_dom_children_and_events_children_match(&node, &events, &expected_child_tags);
}

/// Verify that we can have multiple move to end of siblings patches.
///
/// This test helps check that our patching algorithm will continue to move the correct nodes even
/// as the nodes indices amongst their siblings change.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- multiple_move_to_end_of_siblings_patches
#[wasm_bindgen_test]
fn multiple_move_to_end_of_siblings_patches() {
    let vnode = html! {
      <div>
        <span></span>
        <em></em>
        <img />
        <strong></strong>
        <br />
      </div>
    };

    let (node, mut events) = create_node_and_events_and_append_to_document(vnode);

    percy_dom::patch(
        node.clone(),
        &VirtualNode::element("..."),
        &mut events,
        &vec![
            Patch::MoveToEndOfSiblings {
                parent_old_node_idx: 0,
                siblings_to_move: vec![2],
            },
            Patch::MoveToEndOfSiblings {
                parent_old_node_idx: 0,
                siblings_to_move: vec![4],
            },
        ],
    )
    .unwrap();

    let expected_child_tags = vec!["span", "img", "br", "em", "strong"];
    assert_dom_children_and_events_children_match(&node, &events, &expected_child_tags);
}

/// Verify that the remove children patch removes children.
///
/// _RemoveChildren is used to remove elements in the middle of keyed lists._
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- remove_children_patch_removes_nodes
#[wasm_bindgen_test]
fn remove_children_patch_removes_nodes() {
    let vnode = html! {
      <div>
        <span></span>
        <em></em>
        <img />
        <strong></strong>
        <br />
      </div>
    };

    let (node, mut events) = create_node_and_events_and_append_to_document(vnode);

    percy_dom::patch(
        node.clone(),
        &VirtualNode::element("..."),
        &mut events,
        &vec![Patch::RemoveChildren {
            parent_old_node_idx: 0,
            to_remove: vec![2, 4],
        }],
    )
    .unwrap();

    let expected_child_tags = vec!["span", "img", "br"];
    assert_dom_children_and_events_children_match(&node, &events, &expected_child_tags);
}

/// Verify that we can have multiple remove children patches.
///
/// This test helps check that our patching algorithm will continue to remove the correct node
/// even as if its index amongst its siblings changes between patches.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- multiple_remove_children_patches
#[wasm_bindgen_test]
fn multiple_remove_children_patches() {
    let vnode = html! {
      <div>
        <span></span>
        <em></em>
        <img />
        <strong></strong>
        <br />
      </div>
    };

    let (node, mut events) = create_node_and_events_and_append_to_document(vnode);

    percy_dom::patch(
        node.clone(),
        &VirtualNode::element("..."),
        &mut events,
        &vec![
            Patch::RemoveChildren {
                parent_old_node_idx: 0,
                to_remove: vec![2],
            },
            Patch::RemoveChildren {
                parent_old_node_idx: 0,
                to_remove: vec![4],
            },
        ],
    )
    .unwrap();

    let expected_child_tags = vec!["span", "img", "br"];
    assert_dom_children_and_events_children_match(&node, &events, &expected_child_tags);
}

/// Verify that we can properly patch a list with one keyed element that has not changed.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- one_keyed_element_unchanged_key
#[wasm_bindgen_test]
fn one_keyed_element_unchanged_key() {
    DiffPatchTest {
        desc: "One keyed element unchanged key",
        old: html! {
        <div>
          <em key="a"></em>
        </div>
        },
        new: html! {
        <div>
          <em key="a"></em>
        </div>
        },
        override_expected: None,
    }
    .test();
}

/// Verify that we can properly patch a list with one element where the old and new key are
/// different.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- one_keyed_element_changed_key
#[wasm_bindgen_test]
fn one_keyed_element_changed_key() {
    DiffPatchTest {
        desc: "One keyed element changed key",
        old: html! {
        <div>
          <em key="old-key"></em>
        </div>
        },
        new: html! {
        <div>
          <em key="new-key"></em>
        </div>
        },
        // TODO: Don't add `key` attributes to the DOM element
        override_expected: Some("<div><em key=\"old-key\"></em></div>"),
    }
    .test();
}

/// Verify that we can properly patch a list with one element where the element has the same key
/// but a different tag.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- same_key_different_tag
#[wasm_bindgen_test]
fn same_key_different_tag() {
    DiffPatchTest {
        desc: "Same key different tag.",
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
        override_expected: None,
    }
    .test();
}

/// Verify that we can properly patch a list with two keyed element that have not changed.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- two_keyed_elements_unchanged_keys
#[wasm_bindgen_test]
fn two_keyed_elements_unchanged_keys() {
    DiffPatchTest {
        desc: "Two keyed elements unchanged keys",
        old: html! {
        <div>
          <em key="a"></em>
          <span key="b"></span>
        </div>
        },
        new: html! {
        <div>
          <em key="a"></em>
          <span key="b"></span>
        </div>
        },
        override_expected: None,
    }
    .test();
}

/// Verify that we can properly patch a list with two keyed element have swapped placed.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- two_keyed_elements_swapped
#[wasm_bindgen_test]
fn two_keyed_elements_swapped() {
    DiffPatchTest {
        desc: "Two keyed elements swapped",
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
        override_expected: None,
    }
    .test();
}

/// Verify that we can properly patch a list that has one new keyed element prepended to the list.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- new_keyed_element_prepended
#[wasm_bindgen_test]
fn new_keyed_element_prepended() {
    DiffPatchTest {
        desc: "New keyed element prepended",
        old: html! {
        <div>
          <em key="a"></em>
        </div>
        },
        new: html! {
        <div>
          <span key="b"></span>
          <em key="a"></em>
        </div>
        },
        override_expected: None,
    }
    .test();
}

/// Verify that we can properly patch a list that has one new non-keyed element prepended to the
/// list.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- new_non_keyed_element_prepended
#[wasm_bindgen_test]
fn new_non_keyed_element_prepended() {
    DiffPatchTest {
        desc: "New non-keyed element prepended",
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
        override_expected: None,
    }
    .test();
}

/// Verify that we can properly patch a list that has one new keyed element appended to the list.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- new_keyed_element_appended
#[wasm_bindgen_test]
fn new_keyed_element_appended() {
    DiffPatchTest {
        desc: "New keyed element appended",
        old: html! {
        <div>
          <em key="a"></em>
        </div>
        },
        new: html! {
        <div>
          <em key="a"></em>
          <span key="b"></span>
        </div>
        },
        override_expected: None,
    }
    .test();
}

/// Verify that we can properly patch a list that has one new non-keyed element appended to the
/// list.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- new_non_keyed_element_appended
#[wasm_bindgen_test]
fn new_non_keyed_element_appended() {
    DiffPatchTest {
        desc: "New non keyed element appended",
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
        override_expected: None,
    }
    .test();
}

/// Verify that we can properly patch a list that has one new element inserted into the middle of
/// the list.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- new_keyed_element_inserted_in_middle
#[wasm_bindgen_test]
fn new_keyed_element_inserted_in_middle() {
    DiffPatchTest {
        desc: "New keyed element inserted in middle",
        old: html! {
        <div>
          <em key="a"></em>
          <span key="c"></span>
        </div>
        },
        new: html! {
        <div>
          <em key="a"></em>
          <strong key="b"></strong>
          <span key="c"></span>
        </div>
        },
        override_expected: None,
    }
    .test();
}

/// Verify that we can properly patch a list that has one new element inserted into the middle of
/// the list.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- new_non_keyed_element_inserted_in_middle
#[wasm_bindgen_test]
fn new_non_keyed_element_inserted_in_middle() {
    DiffPatchTest {
        desc: "New non-keyed element inserted in middle",
        old: html! {
        <div>
          <em key="a"></em>
          <span key="c"></span>
        </div>
        },
        new: html! {
        <div>
          <em key="a"></em>
          <strong key="b"></strong>
          <span key="c"></span>
        </div>
        },
        override_expected: None,
    }
    .test();
}

/// Verify that we reverse the order of elements in a keyed list.
///
/// This helps verify that our patches that say to "Move elem X before elem Y" are properly
/// indexed such that as we apply those patches we don't start thing that the "elem Y" in our
/// example is something else.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test keyed_lists -- reverse_list_order
#[wasm_bindgen_test]
fn reverse_list_order() {
    DiffPatchTest {
        desc: "Reverse list order",
        old: html! {
        <div>
          <em key="a"></em>
          <strong key="b"></strong>
          <span key="c"></span>
          <span key="d"></span>
          <span key="e"></span>
        </div>
        },
        new: html! {
        <div>
          <span key="e"></span>
          <span key="d"></span>
          <span key="c"></span>
          <strong key="b"></strong>
          <em key="a"></em>
        </div>
        },
        override_expected: None,
    }
    .test();
}

/// Some of our patches re-order siblings.
///
/// Here we test that a dom node and its corresponding events node both have the same number of
/// children in the same order with the same events ids.
///
/// This confirms that our patches properly re-order both the DOM node siblings and the event node
/// siblings.
///
/// We assert that all of the expected tags are unique so that we are forced to use different
/// tags names for the siblings, which makes our tests easier to reason about.
fn assert_dom_children_and_events_children_match(
    parent_dom_node: &Node,
    events: &VirtualEvents,
    expected_child_tags: &[&'static str],
) {
    let unique: HashSet<&'static str> = expected_child_tags.iter().map(|t| *t).collect();
    assert_eq!(unique.len(), expected_child_tags.len());

    let parent_events_node = events.root();
    let parent_events_node = parent_events_node.borrow();

    let dom_children = parent_dom_node
        .dyn_ref::<web_sys::Element>()
        .unwrap()
        .children();
    let mut next_child = parent_events_node.as_element().unwrap().first_child();

    assert_eq!(dom_children.length() as usize, expected_child_tags.len());

    for (idx, tag) in expected_child_tags.iter().enumerate() {
        let dom_child: web_sys::Element =
            dom_children.item(idx as u32).unwrap().dyn_into().unwrap();
        assert_eq!(dom_child.tag_name().to_lowercase().as_str(), *tag);

        let dom_child_events_id =
            js_sys::Reflect::get(&dom_child, &ELEMENT_EVENTS_ID_PROP.into()).unwrap();
        let dom_child_events_id = dom_child_events_id.as_string().unwrap();
        let dom_child_events_id =
            dom_child_events_id.trim_start_matches(&events.events_id_props_prefix().to_string());
        let dom_child_events_id: u32 = dom_child_events_id.parse().unwrap();

        let events_child = next_child.unwrap();
        let events_child_events_id = events_child.borrow().as_element().unwrap().events_id();

        assert_eq!(dom_child_events_id, events_child_events_id.get());

        next_child = events_child.borrow().next_sibling().cloned();
    }

    assert!(next_child.is_none());
}
