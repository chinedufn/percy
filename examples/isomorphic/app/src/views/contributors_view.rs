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
        let nav_bar = NavBarView::new(ActivePage::Contributors).render();

        let store = self.store.borrow();
        let contributors = store.contributors().to_owned();
        let contributors_list: Vec<VirtualNode> = match contributors {
            Some(contributors) => contributors
                .iter()
                .filter(|c| c.login != "invalid-email-address".to_string())
                .map(|contributor| {
                    html! {
                        <li>
                            <a
                                href=contributor.html_url.to_string()
                                target="_blank"
                            >
                                { contributor.login.to_string() }
                            </a>
                        </li>
                    }
                })
                .collect(),
            None => vec![VirtualNode::text("Loading...")],
        };

        html! {
            <div>
                { nav_bar }
                <div>
                    <ul>
                        { contributors_list }
                    </ul>
                </div>
            </div>
        }
    }
}
