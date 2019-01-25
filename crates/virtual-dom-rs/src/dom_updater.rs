//! Diff virtual-doms and patch the real DOM

use crate::diff::diff;
use crate::patch::patch;
use crate::patch::Patch;
use std::collections::HashMap;
use std::rc::Rc;
use virtual_node::DynClosure;
use virtual_node::VirtualNode;
use wasm_bindgen::JsValue;
use web_sys::*;

type ActiveClosures = HashMap<u32, Vec<DynClosure>>;

/// Used for keeping a real DOM node up to date based on the current VirtualNode
/// and a new incoming VirtualNode that represents our latest DOM state.
pub struct DomUpdater {
    current_vdom: VirtualNode,
    /// The closures that are currently attached to elements in the page.
    ///
    /// We keep these around so that they don't get dropped (and thus stop working);
    ///
    /// TODO: Drop them when the element is no longer in the page
    pub active_closures: ActiveClosures,
    root_node: web_sys::Element,
}

impl DomUpdater {
    /// Create a new DomUpdater.
    ///
    /// A root Element will be created but not added to your DOM.
    pub fn new(current_vdom: VirtualNode) -> DomUpdater {
        let root_node = current_vdom.create_element();

        let active_closures = root_node.closures;
        let root_node = root_node.element;

        web_sys::console::log_1(&format!("closures {}", active_closures.len()).into());

        DomUpdater {
            current_vdom,
            active_closures,
            root_node,
        }
    }

    /// Create a new DomUpdater.
    ///
    /// A root Element will be created and appended to your passed in mount element.
    pub fn new_append_to_mount(current_vdom: VirtualNode, mount: &Element) -> DomUpdater {
        let root_node = current_vdom.create_element();

        let active_closures = root_node.closures;
        let root_node = root_node.element;

        mount.append_child(&root_node);

        DomUpdater {
            current_vdom,
            active_closures,
            root_node,
        }
    }

    /// Create a new DomUpdater.
    ///
    /// A root Element will be created and it will replace your passed in mount element.
    pub fn new_replace_mount(current_vdom: VirtualNode, mount: Element) -> DomUpdater {
        let root_node = current_vdom.create_element();

        let active_closures = root_node.closures;
        let root_node = root_node.element;

        mount.replace_with_with_node_1(&root_node);

        DomUpdater {
            current_vdom,
            active_closures,
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

        self.current_vdom = new_vdom;
    }

    /// Return the root node of your application, the highest ancestor of all other nodes in
    /// your real DOM tree.
    pub fn root_node(&self) -> Element {
        // Note that we're cloning the web_sys::Element, not the DOM element.
        // So we're effectively cloning a pointer here, which is fast.
        self.root_node.clone()
    }
}

impl DomUpdater {
    fn update_active_closures(&mut self, patches: &Vec<Patch>) {
        for patch in patches.iter() {}
    }
}
