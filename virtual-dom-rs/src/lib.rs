//! virtual-dom-rs provides a virtual dom implementation as well as an `html!` macro
//! that you can use to generate a virtual dom.
//!
//! The virtual dom works on both the client and server. On the client we'll render
//! to an `HtmlElement`, and on the server we render to a `String`.

#![deny(missing_docs)]
#![feature(use_extern_macros)]

extern crate wasm_bindgen;

// Used so that `html!` calls work when people depend on this crate since `html!` needs
// access to `Closure` when creating event handlers.
pub use wasm_bindgen::prelude::Closure;

pub extern crate web_sys;
pub use web_sys::*;

#[macro_use]
pub mod html_macro;
pub use html_macro::*;

pub mod virtual_node;
pub use virtual_node::*;

mod diff;
pub use diff::*;

mod patch;
pub use patch::*;
