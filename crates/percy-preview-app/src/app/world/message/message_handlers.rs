use super::Msg;
use crate::app::World;
use app_world::AppWorld;
use app_world::AppWorldWrapper;

mod set_path_message_handler;
mod set_route_data_provider_message_handler;

impl AppWorld for World {
    type Message = Msg;

    fn msg(&mut self, message: Self::Message, wrap: AppWorldWrapper<Self>) {
        match message {
            Msg::AttachRouteDataProvider => self.set_route_data_provider_message_handler(wrap),
            Msg::SetPath(new_path) => {
                self.set_path_message_handler(new_path, wrap);
            }
        };

        if self.state.rendering_enabled {
            let render_fn = self.resources.render_fn.clone();
            self.resources
                .async_task_spawner
                .spawn(Box::pin(async move { (render_fn.lock().unwrap())() }));
        }
    }
}
