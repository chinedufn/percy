//! Diff virtual-doms and patch the real DOM

use std::collections::HashMap;
use std::rc::Rc;
use virtual_node::VirtualNode;
use wasm_bindgen::JsValue;
use web_sys::*;
use crate::diff::diff;
use crate::patch::patch;

type ActiveClosures = HashMap<u32, Rc<Box<dyn AsRef<JsValue>>>>;

/// Used for keeping a real DOM node up to date based on the current VirtualNode
/// and a new incoming VirtualNode that represents our latest DOM state.
pub struct DomUpdater {
    current_vdom: VirtualNode,
    active_closures: ActiveClosures,
    root_node: web_sys::Element,
}

impl DomUpdater {
    /// Create a new DomUpdater.
    ///
    /// A root Element will be created but not added to your DOM.
    pub fn new(current_vdom: VirtualNode) -> DomUpdater {
        let root_node = current_vdom.create_element();

        DomUpdater {
            current_vdom,
            active_closures: HashMap::new(),
            root_node,
        }
    }

    /// Create a new DomUpdater.
    ///
    /// A root Element will be created and appended to your passed in mount element.
    pub fn new_append_to_mount(current_vdom: VirtualNode, mount: Element) -> DomUpdater {
        let root_node = current_vdom.create_element();

        mount.append_child(&root_node);

        DomUpdater {
            current_vdom,
            active_closures: HashMap::new(),
            root_node,
        }
    }

    /// Create a new DomUpdater.
    ///
    /// A root Element will be created and it will replace your passed in mount element.
    ///
    /// # Panics
    ///
    /// - If the mount does not have a parent element.
    ///    You need a parent element to replace an element.
    pub fn new_replace_mount(current_vdom: VirtualNode, mount: Element) -> DomUpdater {
        let root_node = current_vdom.create_element();

        let parent_node = mount.parent_node().expect("Parent node");

        parent_node
            .replace_child(&root_node, &mount)
            .expect("Replaced parent node");

        DomUpdater {
            current_vdom,
            active_closures: HashMap::new(),
            root_node,
        }
    }
}

impl DomUpdater {
    /// Diff the current virtual dom with the new virtual dom that is being passed in.
    ///
    /// Then use that diff to patch the real DOM in the user's browser so that they are
    /// seeing the latest state of the application.
    pub fn update(&mut self, new_vdom: VirtualNode) {
        let patches = diff(&self.current_vdom, &new_vdom);

        patch(self.root_node.clone(), &patches);
    }
}
