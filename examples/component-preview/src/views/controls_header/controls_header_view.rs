use percy_dom::prelude::*;

pub struct ControlsHeaderView {}

impl View for ControlsHeaderView {
    fn render(&self) -> VirtualNode {
        html! {
            <div> This is the component where you can control things like pausing the watcher </div>
        }
    }
}

#[cfg(feature = "preview")]
pub mod preview {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    use percy_preview::{Preview, Rerender};

    pub fn controls_header_preview(_rerender: Rerender) -> Preview {
        let render = move || {
            let view = ControlsHeaderView {};

            html! {
                <div> { view } </div>
            }
        };

        Preview::new("Controls Header View", Rc::new(RefCell::new(render)))
    }
}
