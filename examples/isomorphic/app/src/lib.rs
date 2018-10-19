#[macro_use]
extern crate virtual_dom_rs;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::cell::RefCell;
use std::rc::Rc;
pub use virtual_dom_rs::virtual_node::VirtualNode;

mod store;
pub use crate::store::*;

mod state;
pub use crate::state::*;

pub struct App {
    pub state: Rc<RefCell<State>>,
}

impl App {
    pub fn new(count: u32) -> App {
        App {
            state: Rc::new(RefCell::new(State::new(count))),
        }
    }

    // TODO: Just use `new(config: AppConfig)` and pass in state json Option
    pub fn from_state_json(json: &str) -> App {
        App {
            state: Rc::new(RefCell::new(State::from_json(json))),
        }
    }
}

impl App {
    pub fn render(&self) -> VirtualNode {
        #[allow(unused_variables)] // Compiler doesn't see it inside html macro
        let state = Rc::clone(&self.state);
        let click_count = self.state.borrow().click_count();

        let click_count = &click_count.to_string();

        let click_component = html! { <strong style="font-size: 30px",>{ click_count }</strong> };

        html! {
        <div>
          <span> { "The button has been clicked: " click_component " times!"} </span>
          <button !onclick=move|| { state.borrow_mut().msg(Msg::Click) },>{ "Click me!" }</button>
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

        assert_eq!(app.state.borrow().click_count(), 5);
        app.state.borrow_mut().msg(Msg::Click);
        assert_eq!(app.state.borrow().click_count(), 6);
    }
}
