#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate virtual_dom_rs;

use virtual_dom_rs::prelude::*;

pub use virtual_dom_rs::VirtualNode;

use router_rs::prelude::*;

use serde;

#[macro_use]
extern crate serde_derive;

use serde_json;

use std::cell::RefCell;
use std::rc::Rc;

mod store;
pub use crate::store::*;

mod state;
pub use crate::state::*;
use crate::views::*;
use std::collections::HashMap;

mod views;

pub struct App {
    pub store: Rc<RefCell<Store>>,
    router: Router,
}

impl App {
    pub fn new(count: u32) -> App {
        let state = State::new(count);
        let store = Rc::new(RefCell::new(Store::new(state)));

        let router = make_router(Rc::clone(&store));

        App { store, router }
    }

    // TODO: Just use `new(config: AppConfig)` and pass in state json Option
    pub fn from_state_json(json: &str) -> App {
        let state = State::from_json(json);
        let store = Rc::new(RefCell::new(Store::new(state)));

        let router = make_router(Rc::clone(&store));

        App { store, router }
    }
}

impl App {
    pub fn render(&self) -> VirtualNode {
        #[allow(unused_variables)] // Compiler doesn't see it inside html macro
        let store = Rc::clone(&self.store);

        self.router
            .view(self.store.borrow().path())
            .unwrap()
            .render()
    }
}

fn make_router(store: Rc<RefCell<Store>>) -> Router {
    let mut router = Router::default();

    let store_clone = Rc::clone(&store);

    let param_types = HashMap::new();
    let home_route = Route::new(
        "/",
        param_types,
        Box::new(move |params| Box::new(HomeView::new(Rc::clone(&store_clone))) as Box<View>),
    );

    router.add_route(home_route);

    let param_types = HashMap::new();
    let contributors_route = Route::new(
        "/contributors",
        param_types,
        Box::new(move |params| Box::new(ContributorsView::new(Rc::clone(&store))) as Box<View>),
    );
    router.add_route(contributors_route);

    router
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
