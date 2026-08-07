use percy_dom::prelude::*;
use percy_dom::VirtualNodeWebSys;

pub(super) fn render_active_route() -> VirtualNodeWebSys {
    html! {
        <div>We will render the active route here</div>
    }
}
