use app_world::AppWorldWrapper;

use crate::app::World;

impl World {
    pub(super) fn set_path_message_handler(
        &mut self,
        new_path: String,
        wrap: AppWorldWrapper<Self>,
    ) {
        let active_path = &self.state.active_path;

        if &new_path == active_path {
            return;
        }

        self.state.active_path = new_path.clone();
        (self.resources.after_path_change)(&new_path);

        self.resources
            .async_task_spawner
            .spawn(Box::pin(async move {
                if let Some(route) = wrap
                    .read()
                    .resources
                    .router
                    .matching_route_handler(&new_path)
                {
                    route.on_visit(&new_path);
                }
            }));
    }
}
