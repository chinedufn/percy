use crate::async_task_spawner::AsyncTaskSpawner;
use percy_preview::Preview;
use std::sync::Arc;

/// Configuration for a PercyPreviewApp.
pub(crate) struct AppConfig {
    /// See [`AsyncTaskSpawner`]
    pub async_task_spawner: Arc<dyn AsyncTaskSpawner>,
    /// A function to call whenever the application's URL path changes.
    /// Such as when we visit `/components/component-name`
    pub after_path_change: Box<dyn FnMut(&str) -> ()>,
    /// All of the view components previews.
    pub previews: Vec<Preview>,
}
