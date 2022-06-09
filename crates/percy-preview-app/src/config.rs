use crate::async_task_spawner::AsyncTaskSpawner;
use percy_preview::Preview;
use std::sync::Arc;

/// Configuration for the Percy Preview web client.
pub struct WebClientConfig {
    /// See [`AsyncTaskSpawner`]
    pub async_task_spawner: Arc<dyn AsyncTaskSpawner>,
    /// All of the view components previews.
    pub previews: Vec<Preview>,
}
