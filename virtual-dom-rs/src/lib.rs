//!

#![feature(use_extern_macros)]

extern crate wasm_bindgen;
pub use wasm_bindgen::prelude::Closure;

#[macro_use]
pub mod html_macro;
pub use html_macro::*;

pub mod virtual_node;
pub use virtual_node::*;

// TODO: Replace with web-sys crate when it gets released
pub mod webapis;

mod diff;
pub use diff::*;

mod patch;
pub use patch::*;
