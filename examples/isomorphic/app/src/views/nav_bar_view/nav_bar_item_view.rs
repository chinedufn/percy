use crate::store::Store;
use crate::Msg;
use css_rs_macro::css;
use std::cell::RefCell;
use std::rc::Rc;
use virtual_dom_rs::prelude::*;

pub struct NavBarItemView {
    path: &'static str,
    text: &'static str,
    style: &'static str,
    store: Rc<RefCell<Store>>,
}

impl NavBarItemView {
    pub fn new(
        store: Rc<RefCell<Store>>,
        path: &'static str,
        text: &'static str,
        style: &'static str,
    ) -> NavBarItemView {
        NavBarItemView {
            store,
            path,
            text,
            style,
        }
    }
}

impl View for NavBarItemView {
    fn render(&self) -> VirtualNode {
        let store = Rc::clone(&self.store);

        let path = self.path;

        let text = VirtualNode::text(self.text);

        html! {
            <span
                style=self.style
                class=NAV_BAR_ITEM_CSS
                onclick=move |_ev: u32| {
                  store.borrow_mut().msg(&Msg::Path(path.to_string()));
                }
            >
              { text }
            </span>
        }
    }
}

static NAV_BAR_ITEM_CSS: &'static str = css! {"
:host {
    border-bottom: solid transparent 3px;
    cursor: pointer;
}

:host:hover {
    border-bottom: solid white 3px;
}
"};
