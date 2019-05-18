use css_rs_macro::css;
use virtual_dom_rs::prelude::*;

pub struct NavBarItemView {
    path: &'static str,
    text: &'static str,
    style: &'static str,
}

impl NavBarItemView {
    pub fn new(path: &'static str, text: &'static str, style: &'static str) -> NavBarItemView {
        NavBarItemView { path, text, style }
    }
}

impl View for NavBarItemView {
    fn render(&self) -> VirtualNode {
        html! {
            <a
             href=self.path
             style=self.style
             class=NAV_BAR_ITEM_CSS
            >
              { self.text }
            </a>
        }
    }
}

static NAV_BAR_ITEM_CSS: &'static str = css! {"
:host {
    border-bottom: solid transparent 3px;
    cursor: pointer;
    color: white;
    text-decoration: none;
}

:host:hover {
    border-bottom: solid white 3px;
}
"};
