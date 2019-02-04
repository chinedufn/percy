//! Kept in its own file to more easily import into the book

use console_error_panic_hook;
use virtual_dom_rs::prelude::*;

/// A test case that both diffing and patching are working in a real browser
pub struct DiffPatchTest<'a> {
    pub desc: &'static str,
    pub old: VirtualNode,
    pub new: VirtualNode,
    pub override_expected: Option<&'a str>,
}

impl<'a> DiffPatchTest<'a> {
    pub fn test(&mut self) {
        console_error_panic_hook::set_once();

        let document = web_sys::window().unwrap().document().unwrap();

        // Add our old node into the DOM
        let root_node = self.old.create_element().element;
        // Clone since virtual_dom_rs::patch takes ownership of the root node.
        let patched_root_node = root_node.clone();

        let patches = virtual_dom_rs::diff(&self.old, &self.new);

        // Patch our root node. It should now look like `self.new`
        virtual_dom_rs::patch(root_node, &patches);

        let expected_outer_html = if let Some(ref expected) = self.override_expected {
            expected.to_string()
        } else {
            self.new.to_string()
        };

        assert_eq!(
            &patched_root_node.outer_html(),
            &expected_outer_html,
            "{}",
            self.desc
        );
    }
}
