use crate::app::World;
use app_world::AppWorldWrapper;
use percy_dom::prelude::*;

pub(super) fn render_app(app: &AppWorldWrapper<World>) -> VirtualNode {
    let path = app.read().active_path.clone();

    let view = app.read().resources.router.view(&path);

    if let Some(view) = view {
        view
    } else {
        html! {
            <div>404 page here</div>
        }
    }
}
