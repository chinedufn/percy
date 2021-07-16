use crate::{AppState, AppStateWrapper};
use std::ops::{Deref, DerefMut};

/// Allows us to optionally store sent `Msg`'s instead of passing them on to state.
///
/// This is useful for testing that an event handler dispatched the `Msg`'s that we expected it to.
pub struct StateWithMessageBuffer<S: AppState> {
    state: S,
    message_buffer: Vec<S::Message>,
    capture_messages: bool,
}

impl<S: AppState> StateWithMessageBuffer<S> {
    /// Create a new state and message buffer.
    pub fn new(state: S) -> Self {
        StateWithMessageBuffer {
            state,
            message_buffer: Vec::new(),
            capture_messages: false,
        }
    }
}

impl<S: AppState> AppStateWrapper<S> {
    /// Set whether or not messages get pushed to the message buffer.
    pub fn set_capture_messages(&mut self, capture: bool) {
        self.state.write().unwrap().capture_messages = capture;
    }
}

impl<S: AppState> StateWithMessageBuffer<S> {
    /// After calling `StateWithMessageBuffer.capture_messages(true)`, all .msg() calls will push to this
    /// buffer.
    pub fn message_buffer(&self) -> &Vec<S::Message> {
        &self.message_buffer
    }
}

impl<S: AppState> AppState for StateWithMessageBuffer<S> {
    type Message = S::Message;

    fn msg(&mut self, message: Self::Message) {
        if self.capture_messages {
            self.message_buffer.push(message);
        } else {
            self.state.msg(message);
        }
    }
}

impl<S: AppState> Deref for StateWithMessageBuffer<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<S: AppState> DerefMut for StateWithMessageBuffer<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}
