pub use std::cell::RefCell;
use std::fmt;
pub use std::rc::Rc;

use web_sys;
use web_sys::*;

use js_sys::Function;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

#[derive(Default)]
/// We only support event handlers on wasm32 targets at this time. If you have a use case that
/// needs them elsewhere please open an issue!
pub struct BrowserEvents {
    /// https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.HtmlElement.html#method.oninput
    pub oninput: RefCell<Option<Closure<FnMut(InputEvent) -> ()>>>,
}

impl PartialEq for BrowserEvents {
    fn eq(&self, _rhs: &Self) -> bool {
        true
    }
}

impl fmt::Debug for BrowserEvents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", "TODO: browser event debug implementation")
    }
}

