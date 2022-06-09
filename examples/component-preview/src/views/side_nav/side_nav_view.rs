use percy_dom::prelude::*;

pub struct SideNavView {}

impl View for SideNavView {
    fn render(&self) -> VirtualNode {
        html! {
            <div>
                <div>A side nav with some buttons</div>
                <div>
                    <span>First Button</span>
                </div>
                <div>
                    <span>Second Button</span>
                </div>
            </div>
        }
    }
}

#[cfg(feature = "preview")]
pub mod preview {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    use percy_preview::{Preview, Rerender};

    pub fn side_nav_preview(_rerender: Rerender) -> Preview {
        let render = move || {
            let view = SideNavView {};

            html! {
                <div> { view } </div>
            }
        };

        Preview::new("Side Nav", Rc::new(RefCell::new(render)))
    }
}
