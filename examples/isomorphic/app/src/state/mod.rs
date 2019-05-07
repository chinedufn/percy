use serde::{Serialize, Deserialize};
use serde_json;
use std::cell::Cell;
use std::rc::Rc;

mod msg;
pub use self::msg::Msg;

#[derive(Serialize, Deserialize)]
pub struct State {
    click_count: Rc<Cell<u32>>,
    path: String,
}

impl State {
    pub fn new(count: u32) -> State {
        State {
            path: "/".to_string(),
            click_count: Rc::new(Cell::new(count)),
        }
    }

    pub fn from_json(state_json: &str) -> State {
        serde_json::from_str(state_json).unwrap()
    }
}

impl State {
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl State {
    pub fn msg(&mut self, msg: &Msg) {
        match msg {
            Msg::Click => self.increment_click(),
            Msg::SetPath(path) => self.set_path(path.to_string()),
        };
    }

    pub fn click_count(&self) -> u32 {
        self.click_count.get()
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl State {
    fn increment_click(&mut self) {
        self.click_count.set(self.click_count.get() + 1);
    }

    fn set_path(&mut self, path: String) {
        self.path = path;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_deserialize() {
        let state_json = r#"{"click_count":5,"path":"/"}"#;

        let state = State::from_json(state_json);

        assert_eq!(state.click_count(), 5);

        assert_eq!(&state.to_json(), state_json);
    }
}
