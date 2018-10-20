use crate::store::Store;
use crate::Msg;
use css_rs_macro::css;
use std::cell::RefCell;
use std::rc::Rc;
use virtual_dom_rs::prelude::*;

pub struct NavBarView {
    active_page: ActivePage,
    store: Rc<RefCell<Store>>,
}

impl NavBarView {
    pub fn new(active_page: ActivePage, store: Rc<RefCell<Store>>) -> NavBarView {
        NavBarView { active_page, store }
    }
}

pub enum ActivePage {
    Home,
    Contributors,
}

impl View for NavBarView {
    fn render(&self) -> VirtualNode {
        let store = self.store.borrow();
        let path = store.path();
        let path = path.clone();

        let store = Rc::clone(&self.store);

        let home = html! {
            <span !onclick=move || {
                    store.borrow_mut().msg(&Msg::Path("/".to_string()));
                },
            >
              { "Home" }
              { path }
            </span>
        };

        let store = Rc::clone(&self.store);
        let contributors = html! {
            <span !onclick=move || {
                store.borrow_mut().msg(&Msg::Path("/contributors".to_string()));
            },>
                { "Contributors" }
            </span>
        };

        html! {
        <div class=*NavBarCSS,>
            { home }
            { contributors }
        </div>
        }
    }
}

static NavBarCSS: &'static str = css! {"
:host {
    background-color: red;
}
"};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_nav() {
        let nav_bar = ActivePage::new(ActivePage::Home);
    }
}
