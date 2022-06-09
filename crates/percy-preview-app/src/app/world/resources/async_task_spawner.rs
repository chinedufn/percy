//! Spawn async tasks.

use std::future::Future;
use std::pin::Pin;

/// Used to run an async function.
///
/// In the browser this will typically schedule the future to be called on the next tick of
/// the JS microtask queue.
///
/// Outside of JS this will often spawn the task in a thread.
pub trait AsyncTaskSpawner: Send + Sync {
    /// Run an async function, typically in another thread or, if in the browser, on another tick
    /// of the JS event loop.
    fn spawn(&self, task: AsyncFnToSpawn);
}

/// The async task to run.
pub type AsyncFnToSpawn = Pin<Box<dyn Future<Output = ()> + 'static>>;
