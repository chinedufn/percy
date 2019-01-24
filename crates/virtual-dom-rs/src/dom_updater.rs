//! Diff virtual-doms and patch the real DOM

use wasm_bindgen::JsValue;
use std::rc::Rc;
use std::collections::HashMap;
use virtual_node::VirtualNode;

type ActiveClosures = HashMap<u32, Rc<Box<dyn AsRef<JsValue>>>>;

/// Used for keeping a real DOM node up to date based on the current VirtualNode
/// and a new incoming VirtualNode that represents our latest DOM state.
pub struct DomUpdater {
    current_vom: VirtualNode,
    active_closures: ActiveClosures
}