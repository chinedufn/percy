#[macro_use]
extern crate virtual_dom_rs;

use serde;

#[macro_use]
extern crate serde_derive;

use serde_json;

use std::cell::RefCell;
use std::rc::Rc;
pub use virtual_dom_rs::virtual_node::VirtualNode;

mod store;
pub use crate::store::*;

mod state;
pub use crate::state::*;

mod views;

pub struct App {
    pub store: Rc<RefCell<Store>>,
}

impl App {
    pub fn new(count: u32) -> App {
        let state = State::new(count);

        App {
            store: Rc::new(RefCell::new(Store::new(state))),
        }
    }

    // TODO: Just use `new(config: AppConfig)` and pass in state json Option
    pub fn from_state_json(json: &str) -> App {
        let state = State::from_json(json);

        App {
            store: Rc::new(RefCell::new(Store::new(state))),
        }
    }
}

impl App {
    pub fn render(&self) -> VirtualNode {
        #[allow(unused_variables)] // Compiler doesn't see it inside html macro
        let store = Rc::clone(&self.store);

        let click_count = self.store.borrow().click_count();
        let click_count = &click_count.to_string();

        let click_component = html! { <strong style="font-size: 30px",>{ click_count }</strong> };

        html! {
        <div>
          <span> { "The button has been clicked: " click_component " times!"} </span>
          <button !onclick=move|| { store.borrow_mut().msg(&Msg::Click) },>{ "Click me!" }</button>
          <div> { "In this time " click_count " rustaceans have been born." } </div>
        </div>
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_msg() {
        let app = App::new(5);

        assert_eq!(app.store.borrow().click_count(), 5);
        app.store.borrow_mut().msg(&Msg::Click);
        assert_eq!(app.store.borrow().click_count(), 6);
    }
}
