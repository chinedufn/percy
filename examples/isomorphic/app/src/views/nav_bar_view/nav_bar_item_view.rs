use percy_css_macro::css;
use percy_dom::prelude::*;
use percy_dom::VirtualNodeWebSys;

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

impl View<web_sys::Window> for NavBarItemView {
    fn render(&self) -> VirtualNodeWebSys {
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
