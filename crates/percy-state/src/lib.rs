//! Used to manage application state.

#![deny(missing_docs)]

use std::sync::{Arc, RwLock, RwLockReadGuard};

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
#[derive(Clone)]
pub struct AppStateWrapper<S: AppState>(Arc<RwLock<S>>);

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
    pub fn new(state: S) -> Self {
        Self(Arc::new(RwLock::new(state)))
    }

    /// Acquire write access to the AppState then send a message.
    pub fn msg(&mut self, msg: S::Message) {
        self.0.write().unwrap().msg(msg);
    }

    /// Acquire read access to AppState.
    pub fn read(&self) -> RwLockReadGuard<'_, S> {
        self.0.read().unwrap()
    }
}
