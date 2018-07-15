#[macro_use]
extern crate virtual_dom_rs;

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use virtual_dom_rs::virtual_node::VirtualNode;

pub struct App {
    pub state: Rc<RefCell<State>>,
}

impl App {
    pub fn new() -> App {
        App {
            state: Rc::new(RefCell::new(State::new())),
        }
    }
}

impl App {
    pub fn render(&self) -> VirtualNode {
        let state = Rc::clone(&self.state);
        let click_count = self.state.borrow().click_count();
        let click_count = click_count.to_string();

        let click_count = html! { <strong style="font-size: 30px",>{ click_count }</strong> };

        html! {
        <div>
          <span> { "The button has been clicked: " click_count  " times!"} </span>
          <button !onclick=move|| { state.borrow_mut().msg(Msg::Click) },>{ "Click me!" }</button>
        </div>
        }
    }
}

pub struct State {
    click_count: Rc<Cell<u32>>,
}

impl State {
    pub fn new() -> State {
        State {
            click_count: Rc::new(Cell::new(0)),
        }
    }
}

pub enum Msg {
    Click,
}

impl State {
    pub fn msg(&mut self, msg: Msg) {
        match msg {
            Msg::Click => self.increment_click(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_msg() {
        let mut app = App::new();

        assert_eq!(app.state.borrow().click_count(), 0);
        app.state.borrow_mut().msg(Msg::Click);
        assert_eq!(app.state.borrow().click_count(), 1);
    }
}
