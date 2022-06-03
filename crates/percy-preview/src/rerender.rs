use std::sync::{Arc, Mutex};

type Render = Arc<Mutex<Box<dyn FnMut() -> ()>>>;

/// Allows the preview to trigger a rerender of itself.
///
/// For example, a preview for a component with a button that increments
/// a counter might trigger a rerender whenever the button is clicked.
///
/// # Clone
///
/// Clones share the same render function.
#[derive(Clone)]
pub struct Rerender {
    render: Render,
}

impl Rerender {
    /// Create a new Rerender
    pub fn new(render: Render) -> Self {
        Rerender {
            render: Arc::new(Mutex::new(Box::new(|| {}))),
        }
    }

    /// Rerender the Percy Preview App.
    pub fn rerender(&self) {
        (self.render.lock().unwrap())()
    }

    /// Get the inner render function.
    pub fn set_render_fn(&self, render_fn: Box<dyn FnMut() -> ()>) {
        *self.render.lock().unwrap() = render_fn;
    }
}
