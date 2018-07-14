//!

#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

#[macro_use]
extern crate wasm_bindgen;

pub mod virtual_node;
pub use virtual_node::*;

#[macro_use]
pub mod html_macro;
pub use html_macro::*;

// TODO: Replace with web-sys crate when it gets released
mod webapis;
