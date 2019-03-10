#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate serde_derive;

pub use crate::state::*;
pub use crate::store::*;
use crate::views::*;
use router_rs::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use virtual_dom_rs::prelude::*;
pub use virtual_dom_rs::VirtualNode;

mod state;
mod store;
mod views;

pub struct App {
    pub store: Rc<RefCell<Store>>,
    router: Router,
}

impl App {
    pub fn new(count: u32, path: String) -> App {
        let state = State::new(count);
        let store = Rc::new(RefCell::new(Store::new(state)));

        store.borrow_mut().msg(&Msg::Path(path));

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

        self.router.view(self.store.borrow().path()).unwrap()
    }
}

#[route(path = "/")]
fn home_route(store: Provided<Rc<RefCell<Store>>>) -> VirtualNode {
    HomeView::new(Rc::clone(&store)).render()
}

#[route(path = "/contributors")]
fn contributors_route(store: Provided<Rc<RefCell<Store>>>) -> VirtualNode {
    ContributorsView::new(Rc::clone(&store)).render()
}

fn make_router(store: Rc<RefCell<Store>>) -> Router {
    let mut router = Router::default();

    router.provide(store);

    router.set_route_handlers(create_routes![home_route, contributors_route]);

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
