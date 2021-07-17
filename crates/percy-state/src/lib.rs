//! Used to manage application state.

#![deny(missing_docs)]

use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub use self::world_with_message_buffer::*;

mod world_with_message_buffer;

/// A function that can render the application and update the DOM.
pub type RenderFn = Arc<Mutex<Box<dyn FnMut() -> ()>>>;

/// Holds application state and resources, and will trigger a re-render after .msg() calls.
///
/// # Cloning
///
/// Cloning an `AppWorldWrapper` is a very cheap operation.
///
/// It can be useful to clone `AppWorldWrapper`'s in order to pass the world into event handler
/// closures.
///
/// All clones hold pointers to the same inner state.
pub struct AppWorldWrapper<W: AppWorld> {
    world: Arc<RwLock<WorldWithMessageBuffer<W>>>,
    render_fn: RenderFn,
}

/// Defines how messages that indicate that something has happened get sent to the World.
pub trait AppWorld: Sized {
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
    fn msg(&mut self, message: Self::Message, world_wrapper: AppWorldWrapper<Self>);
}

impl<W: AppWorld> AppWorldWrapper<W> {
    /// Create a new AppWorldWrapper.
    pub fn new(world: W, render_fn: RenderFn) -> Self {
        Self {
            world: Arc::new(RwLock::new(WorldWithMessageBuffer::new(world))),
            render_fn,
        }
    }

    /// Acquire write access to the AppWorld then send a message.
    pub fn msg(&self, msg: W::Message) {
        self.world
            .write()
            .unwrap()
            .message_maybe_capture(msg, self.clone());

        (self.render_fn.lock().unwrap())();
    }

    /// Acquire read access to AppWorld.
    pub fn read(&self) -> RwLockReadGuard<'_, WorldWithMessageBuffer<W>> {
        self.world.read().unwrap()
    }
}

impl<S: AppWorld> Clone for AppWorldWrapper<S> {
    fn clone(&self) -> Self {
        AppWorldWrapper {
            world: Arc::clone(&self.world),
            render_fn: Arc::clone(&self.render_fn),
        }
    }
}
