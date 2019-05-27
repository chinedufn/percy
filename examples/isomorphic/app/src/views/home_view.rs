use crate::store::Store;
use crate::views::nav_bar_view::ActivePage;
use crate::views::nav_bar_view::NavBarView;
use crate::Msg;

use virtual_dom_rs::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

pub struct HomeView {
    store: Rc<RefCell<Store>>,
}

impl HomeView {
    pub fn new(store: Rc<RefCell<Store>>) -> HomeView {
        HomeView { store }
    }
}

impl View for HomeView {
    fn render(&self) -> VirtualNode {
        let nav_bar = NavBarView::new(ActivePage::Home).render();

        let store = Rc::clone(&self.store);

        let click_count = self.store.borrow().click_count();
        let click_count = &*click_count.to_string();

        let click_component = html! { <strong style="font-size: 30px">{ click_count }</strong> };

        html! {
        <div>

          { nav_bar }

          <span> The button has been clicked: { click_component } times! </span>
          <button onclick=move|_: web_sys::Event| { store.borrow_mut().msg(&Msg::Click) }>
            Click me!
          </button>
          <div> In this time Ferris has made { click_count } new friends. </div>
            <svg xmlns="http://www.w3.org/2000/svg">
              <circle cx="50" cy="50" r="50"/>
            </svg>
        </div>
        }
    }
}
