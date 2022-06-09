pub struct WebAsyncTaskSpawner;

#[cfg(feature = "preview")]
mod impl_preview {
    use percy_preview_app::async_task_spawner::{AsyncFnToSpawn, AsyncTaskSpawner};

    impl AsyncTaskSpawner for super::WebAsyncTaskSpawner {
        fn spawn(&self, task: AsyncFnToSpawn) {
            wasm_bindgen_futures::spawn_local(task)
        }
    }
}
