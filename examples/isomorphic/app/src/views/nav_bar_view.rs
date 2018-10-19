use virtual_dom_rs::prelude::*;

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
}

impl View for NavBarView {
    fn render(&self) -> VirtualNode {
        html! {
        <div>
        { "NAV BAR GOES HERE" }
        </div>
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_nav() {
        let nav_bar = ActivePage::new(ActivePage::Home);
    }
}
