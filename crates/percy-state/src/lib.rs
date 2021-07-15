//! Used to manage application state.

#![deny(missing_docs)]

use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};

/// A function that can render the application.
pub type RenderFn = Arc<Mutex<Box<dyn FnMut() -> ()>>>;

/// Holds application state.
///
/// # Cloning
///
/// It can be useful to clone `AppStateWrapper`'s in order to pass state into event handler
/// closures.
///
/// All clones will point to the same inner state.
///
/// Cloning an `AppStateWrapper` is a very cheap operation.
pub struct AppStateWrapper<S: AppState> {
    state: Arc<RwLock<S>>,
    render: RenderFn,
}

/// Application state.
pub trait AppState {
    /// Indicates that something has happened.
    ///
    /// ```
    /// # use std::time::SystemTime;
    /// #[allow(unused)]
    /// enum MyMessageType {
    ///     IncreaseClickCounter,
    ///     SetLastPausedAt(SystemTime)
    /// }
    /// ```
    type Message;

    /// Send a message to the state object.
    /// This will usually lead to a state update
    fn msg(&mut self, message: Self::Message);
}

impl<S: AppState> AppStateWrapper<S> {
    /// Create a new AppStateWrapper.
    pub fn new(state: S, render: RenderFn) -> Self {
        Self {
            state: Arc::new(RwLock::new(state)),
            render,
        }
    }

    /// Acquire write access to the AppState then send a message.
    pub fn msg(&self, msg: S::Message) {
        self.state.write().unwrap().msg(msg);

        (self.render.lock().unwrap())();
    }

    /// Acquire read access to AppState.
    pub fn read(&self) -> RwLockReadGuard<'_, S> {
        self.state.read().unwrap()
    }
}

impl<S: AppState> Clone for AppStateWrapper<S> {
    fn clone(&self) -> Self {
        AppStateWrapper {
            state: Arc::clone(&self.state),
            render: Arc::clone(&self.render),
        }
    }
}
