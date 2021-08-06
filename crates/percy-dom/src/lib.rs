//! percy-dom provides a virtual dom implementation as well as an `html!` macro
//! that you can use to generate a virtual dom.
//!
//! The virtual dom works on both the client and server. On the client we'll render
//! to an `HtmlElement`, and on the server we render to a `String`.

#![deny(missing_docs)]

extern crate wasm_bindgen;

// Used so that `html!` calls work when people depend on this crate since `html!` needs
// access to `Closure` when creating event handlers.
pub use wasm_bindgen::prelude::Closure;
pub use wasm_bindgen::JsCast;

pub extern crate web_sys;
pub use web_sys::*;

pub use virtual_node::*;

mod diff;
pub use crate::diff::*;

mod patch;
pub use crate::patch::*;

// FIXME: Remove this and have users depend on html-macro directly.
//  That way we don't need to re-release this crate whenever we update html-macro.
pub use html_macro::html;

mod dom_updater;
pub use self::dom_updater::DomUpdater;

/// Exports structs and macros that you'll almost always want access to in a virtual-dom
/// powered application
pub mod prelude {
    pub use crate::dom_updater::DomUpdater;
    pub use crate::VirtualNode;
    pub use html_macro::html;
    pub use std::vec::IntoIter;
    pub use virtual_node::{event::*, wrap_closure, EventAttribFn, IterableNodes, View};
    pub use wasm_bindgen::prelude::Closure;
}
