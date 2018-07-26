//!

#![feature(use_extern_macros)]

extern crate wasm_bindgen;
pub use wasm_bindgen::prelude::Closure;

// TODO: Replace with web-sys crate when it gets released
pub extern crate percy_webapis;

#[macro_use]
pub mod html_macro;
pub use html_macro::*;

pub mod virtual_node;
pub use virtual_node::*;


mod diff;
pub use diff::*;

mod patch;
pub use patch::*;
