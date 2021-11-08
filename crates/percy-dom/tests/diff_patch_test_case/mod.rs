//! Kept in its own file to more easily import into the book

use console_error_panic_hook;
use percy_dom::event::EventsByNodeIdx;
use percy_dom::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, Node};

/// A test case that both diffing and patching are working in a real browser
pub struct DiffPatchTest<'a> {
    /// Description of the test case.
    pub desc: &'static str,
    /// The old virtual node.
    pub old: VirtualNode,
    /// The new virtual node.
    pub new: VirtualNode,
    /// By default we generate the expected based on `new.to_string()`. You can
    /// use this field to override the expected HTML after patching.
    pub override_expected: Option<&'a str>,
}

impl<'a> DiffPatchTest<'a> {
    pub fn test(&mut self) {
        console_error_panic_hook::set_once();

        let mut events = EventsByNodeIdx::new();

        // Create a DOM node of the virtual root node
        let root_node: Node = self.old.create_dom_node(0, &mut events);

        // Clone since percy_dom::patch takes ownership of the root node.
        let patched_root_node: Node = root_node.clone();

        // Generate patches
        let patches = percy_dom::diff(&self.old, &self.new);

        // Patch our root node. It should now look like `self.new`
        percy_dom::patch(root_node, &self.new, &mut events, &patches).unwrap();

        // Determine the expected outer HTML
        let expected_outer_html = match self.override_expected {
            Some(ref expected) => expected.to_string(),
            None => self.new.to_string(),
        };

        let actual_outer_html = match patched_root_node.node_type() {
            Node::ELEMENT_NODE => patched_root_node.unchecked_into::<Element>().outer_html(),
            Node::TEXT_NODE => patched_root_node.text_content().unwrap_or("".into()),
            _ => panic!("Unhandled node type"),
        };

        assert_eq!(&actual_outer_html, &expected_outer_html, "{}", self.desc);
    }
}
