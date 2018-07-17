use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde_json;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Serialize, Deserialize)]
pub struct State {
    #[serde(deserialize_with = "deserialize_rc_cell", serialize_with = "serialize_rc_cell")]
    click_count: Rc<Cell<u32>>,
    #[serde(skip)]
    listeners: Vec<Box<Fn() -> ()>>,
}

impl State {
    pub fn new(count: u32) -> State {
        State {
            click_count: Rc::new(Cell::new(count)),
            listeners: vec![],
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
    pub fn subscribe(&mut self, callback: Box<Fn() -> ()>) {
        self.listeners.push(callback)
    }
}

pub enum Msg {
    Click,
}

impl State {
    pub fn msg(&mut self, msg: Msg) {
        match msg {
            Msg::Click => self.increment_click(),
        };

        for callback in self.listeners.iter() {
            callback();
        }
    }

    pub fn click_count(&self) -> u32 {
        self.click_count.get()
    }
}

impl State {
    fn increment_click(&mut self) {
        self.click_count.set(self.click_count.get() + 1);
    }
}
// serde does not support serialize / deserialize rc so we use our own deserializer
fn deserialize_rc_cell<'de, D, T>(deserializer: D) -> Result<Rc<Cell<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Copy,
{
    Ok(Rc::new(Cell::deserialize(deserializer).unwrap()))
}

// serde does not support serialize / deserialize rc so we use our own serializer
fn serialize_rc_cell<T, S>(
    rc_cell: &Rc<Cell<T>>,
    serializer: S,
) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
where
    S: Serializer,
    T: Serialize + Copy,
{
    rc_cell.serialize(serializer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_deserialize() {
        let state_json = r#"{"click_count":5}"#;

        let state = State::from_json(state_json);

        assert_eq!(state.click_count(), 5);

        assert_eq!(&state.to_json(), state_json);
    }
}
