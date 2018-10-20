use crate::state::Msg;
use crate::state::State;

use std::ops::Deref;

pub struct Store {
    state: StateWrapper,
}

impl Store {
    pub fn new(state: State) -> Store {
        Store {
            state: StateWrapper(state),
        }
    }

    pub fn msg(&mut self, msg: &Msg) {
        match msg {
            Msg::Path(path) => self.state.msg(msg),
            _ => self.state.msg(msg),
        }
    }

    pub fn subscribe(&mut self, callback: Box<Fn() -> ()>) {
        self.state.subscribe(callback);
    }
}

impl Deref for Store {
    type Target = State;

    fn deref(&self) -> &<Self as Deref>::Target {
        &self.state
    }
}

struct StateWrapper(State);

impl Deref for StateWrapper {
    type Target = State;

    fn deref(&self) -> &<Self as Deref>::Target {
        &self.0
    }
}

impl StateWrapper {
    fn msg(&mut self, msg: &Msg) {
        self.0.msg(msg)
    }

    fn subscribe(&mut self, callback: Box<Fn() -> ()>) {
        self.0.subscribe(callback);
    }
}
