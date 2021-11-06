//! Diff virtual-doms and patch the real DOM

use crate::diff::diff;
use crate::event::EventsByNodeIdx;
use crate::patch::patch;
use std::collections::HashMap;
use virtual_node::VirtualNode;
use wasm_bindgen::JsValue;
use web_sys::{Element, Node};

mod events;

/// Used for keeping a real DOM node up to date based on the current VirtualNode
/// and a new incoming VirtualNode that represents our latest DOM state.
///
/// Also powers event delegation.
pub struct PercyDom {
    current_vdom: VirtualNode,
    /// The closures that are currently attached to elements in the page.
    /// We keep these around so that they don't get dropped (and thus stop working).
    pub events: EventsByNodeIdx,
    root_node: Node,
    // We hold onto these since if we drop the listener it can no longer be called.
    event_delegation_listeners: HashMap<&'static str, Box<dyn AsRef<JsValue>>>,
}

impl PercyDom {
    /// Create a new `PercyDom`.
    ///
    /// A root `Node` will be created but not added to your DOM.
    pub fn new(current_vdom: VirtualNode) -> PercyDom {
        let mut events = EventsByNodeIdx::new();
        let created_node = current_vdom.create_dom_node(0, &mut events);

        let mut pdom = PercyDom {
            current_vdom,
            root_node: created_node,
            events,
            event_delegation_listeners: HashMap::new(),
        };
        pdom.attach_event_listeners();

        pdom
    }

    /// Create a new `PercyDom`.
    ///
    /// A root `Node` will be created and append (as a child) to your passed
    /// in mount element.
    pub fn new_append_to_mount(current_vdom: VirtualNode, mount: &Element) -> PercyDom {
        let pdom = Self::new(current_vdom);

        mount
            .append_child(&pdom.root_node)
            .expect("Could not append child to mount");

        pdom
    }

    /// Create a new `PercyDom`.
    ///
    /// A root `Node` will be created and it will replace your passed in mount
    /// element.
    pub fn new_replace_mount(current_vdom: VirtualNode, mount: Element) -> PercyDom {
        let pdom = Self::new(current_vdom);

        mount
            .replace_with_with_node_1(&pdom.root_node)
            .expect("Could not replace mount element");

        pdom
    }

    /// Diff the current virtual dom with the new virtual dom that is being passed in.
    ///
    /// Then use that diff to patch the real DOM in the user's browser so that they are
    /// seeing the latest state of the application.
    pub fn update(&mut self, new_vdom: VirtualNode) {
        let patches = diff(&self.current_vdom, &new_vdom);

        patch(
            self.root_node.clone(),
            &new_vdom,
            &mut self.events,
            &patches,
        )
        .unwrap();

        self.current_vdom = new_vdom;
    }

    /// Return the root node of your application, the highest ancestor of all other nodes in
    /// your real DOM tree.
    pub fn root_node(&self) -> Node {
        // Note that we're cloning the `web_sys::Node`, not the DOM element.
        // So we're effectively cloning a pointer here, which is fast.
        self.root_node.clone()
    }
}
