use percy_preview_app::async_task_spawner::{AsyncFnToSpawn, AsyncTaskSpawner};

pub struct WebAsyncTaskSpawner;

impl AsyncTaskSpawner for WebAsyncTaskSpawner {
    fn spawn(&self, task: AsyncFnToSpawn) {
        wasm_bindgen_futures::spawn_local(task)
    }
}
