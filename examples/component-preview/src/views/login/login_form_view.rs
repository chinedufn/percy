use percy_dom::prelude::*;
use percy_dom::VirtualNodeWebSys;

pub struct LoginFormView {}

impl View<web_sys::Window> for LoginFormView {
    fn render(&self) -> VirtualNodeWebSys {
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

        let mut preview = Preview::new("Login Form", Rc::new(RefCell::new(render)));
        preview.set_description(Some(
            "This is the login form preview's description.".to_string(),
        ));

        preview
    }
}
