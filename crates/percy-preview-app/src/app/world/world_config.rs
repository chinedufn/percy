use crate::async_task_spawner::AsyncTaskSpawner;
use percy_preview::Preview;
use percy_router::prelude::Router;
use std::sync::{Arc, Mutex};

pub(crate) struct WorldConfig {
    /// See [`AsyncTaskSpawner`]
    pub async_task_spawner: Arc<dyn AsyncTaskSpawner>,
    /// A function to call whenever the application's URL path changes.
    /// Such as when we visit `/components/component-name`
    pub(crate) after_path_change: Box<dyn FnMut(&str) -> ()>,
    /// All of the view components that can be previewed.
    pub previews: Vec<Preview>,
    /// Used to re-render the application into the DOM.
    pub render: Arc<Mutex<Box<dyn FnMut() -> ()>>>,
    /// See [`Router`].
    pub router: Router,
}
