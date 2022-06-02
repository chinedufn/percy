use percy_preview::Preview;
use percy_router::prelude::Router;
use std::sync::{Arc, Mutex};

pub(crate) struct WorldConfig {
    /// All of the view components that can be previewed.
    pub previews: Vec<Preview>,
    /// Used to re-render the application into the DOM.
    pub render: Arc<Mutex<Box<dyn FnMut() -> ()>>>,
    /// See [`Router`].
    pub router: Router,
}
