use crate::async_task_spawner::AsyncTaskSpawner;
use percy_router::prelude::Router;
use std::sync::{Arc, Mutex};

/// A function used to re-render the application.
pub type RenderFn = Arc<Mutex<Box<dyn FnMut() -> ()>>>;

pub mod async_task_spawner;

/// Application resources.
pub(crate) struct Resources {
    /// A function to call whenever the application's URL path changes.
    /// Such as when we visit `/components/component-name`
    pub after_path_change: Box<dyn FnMut(&str) -> ()>,
    /// See [`AsyncTaskSpawner`]
    pub async_task_spawner: Arc<dyn AsyncTaskSpawner>,
    /// See [`Router`]
    pub router: Router,
    /// See [`RenderFn`]
    pub render_fn: RenderFn,
}
