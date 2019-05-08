use css_rs_macro::css;
use virtual_dom_rs::prelude::*;

mod nav_bar_item_view;
use self::nav_bar_item_view::NavBarItemView;

pub struct NavBarView {
    active_page: ActivePage,
}

impl NavBarView {
    pub fn new(active_page: ActivePage) -> NavBarView {
        NavBarView { active_page }
    }
}

pub enum ActivePage {
    Home,
    Contributors,
}

impl View for NavBarView {
    fn render(&self) -> VirtualNode {
        let home = NavBarItemView::new("/", "Isomorphic Web App", "");
        let contributors =
            NavBarItemView::new("/contributors", "Contributors", "margin-left: auto;");

        html! {
        <div class=NAV_BAR_CSS>
            { home.render() }
            { contributors.render() }
        </div>
        }
    }
}

static NAV_BAR_CSS: &'static str = css! {"
:host {
    align-items: center;
    background: linear-gradient(267deg,#2a38ef,#200994 50%,#1c2dab);
    color: white;
    display: flex;
    font-family: Avenir-Next;
    font-size: 20px;
    font-weight: bold;
    height: 50px;
    padding-left: 30px;
    padding-right: 30px;
}
"};
