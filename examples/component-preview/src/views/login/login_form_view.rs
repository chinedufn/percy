use percy_dom::prelude::*;

pub struct LoginFormView {}

impl View for LoginFormView {
    fn render(&self) -> VirtualNode {
        html! {
            <div>
               This will be a login form
                <div class=css!("display-flex flex-dir-col")>
                    <input type="text" placeholder="username" />
                    <input type="password" placeholder="Password" />
                    <button>Submit</button>
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

    pub fn login_form_preview(_rerender: Rerender) -> Preview {
        let render = move || {
            let view = LoginFormView {};

            html! {
                <div> { view } </div>
            }
        };

        Preview::new("Login Form View", Rc::new(RefCell::new(render)))
    }
}
