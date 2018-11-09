//! virtual-dom-rs provides a virtual dom implementation as well as an `html!` macro
//! that you can use to generate a virtual dom.
//!
//! The virtual dom works on both the client and server. On the client we'll render
//! to an `HtmlElement`, and on the server we render to a `String`.

#![deny(missing_docs)]

extern crate wasm_bindgen;

// Used so that `html!` calls work when people depend on this crate since `html!` needs
// access to `Closure` when creating event handlers.
pub use wasm_bindgen::prelude::Closure;
#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen::JsCast;

pub extern crate web_sys;
pub use web_sys::*;

#[macro_use]
pub mod html_macro;
pub use crate::html_macro::*;

pub mod virtual_node;
pub use crate::virtual_node::*;

#[cfg(target_arch = "wasm32")]
mod diff;
#[cfg(target_arch = "wasm32")]
pub use crate::diff::*;

#[cfg(target_arch = "wasm32")]
mod patch;
#[cfg(target_arch = "wasm32")]
pub use crate::patch::*;

mod view;
pub use crate::view::*;

/// Exports structs and macros that you'll almost always want access to in a virtual-dom
/// powered application
pub mod prelude {
    pub use crate::html_macro::*;
    pub use crate::view::View;
    pub use crate::virtual_node::VirtualNode;
}
