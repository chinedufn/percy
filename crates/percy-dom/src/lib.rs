//! percy-dom provides a virtual dom implementation as well as an `html!` macro
//! that you can use to generate a virtual dom.
//!
//! The virtual dom works on both the client and server. On the client we'll render
//! to an `HtmlElement`, and on the server we render to a `String`.

#![deny(missing_docs)]

extern crate wasm_bindgen;

pub use wasm_bindgen::JsCast;
// Used so that `html!` calls work when people depend on this crate since `html!` needs
// access to `Closure` when creating event handlers.
pub use wasm_bindgen::prelude::Closure;

pub use html_macro::html;
pub use virtual_node::*;

pub use crate::diff::*;
pub use crate::patch::*;

pub use self::pdom::PercyDom;

mod diff;
mod patch;
mod pdom;

/// Exports structs and macros that you'll almost always want access to in a virtual-dom
/// powered application
pub mod prelude {
    // TODO: look through this prelude and remove anything that isn't necessary.

    pub use std::vec::IntoIter;

    pub use wasm_bindgen::prelude::Closure;

    pub use html_macro::html;
    pub use virtual_node::{EventAttribFn, IterableNodes, View};

    pub use crate::pdom::PercyDom;
    pub use crate::VirtualNode;

    // Used by the html-macro crate.
    #[doc(hidden)]
    pub mod __html_macro_helpers__ {
        pub use virtual_node::event;
        pub use web_sys;
    }
}
