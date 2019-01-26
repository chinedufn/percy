use crate::store::Store;
use crate::views::nav_bar_view::ActivePage;
use crate::views::nav_bar_view::NavBarView;
use std::cell::RefCell;
use std::rc::Rc;
use virtual_dom_rs::prelude::*;

pub struct ContributorsView {
    store: Rc<RefCell<Store>>,
}

impl ContributorsView {
    pub fn new(store: Rc<RefCell<Store>>) -> ContributorsView {
        ContributorsView { store }
    }
}

impl View for ContributorsView {
    fn render(&self) -> VirtualNode {
        let nav_bar = NavBarView::new(ActivePage::Contributors, Rc::clone(&self.store)).render();

        html! {
        <div>
            { nav_bar }
            <div>
             Contributors page here
            </div>
        </div>
        }
    }
}
