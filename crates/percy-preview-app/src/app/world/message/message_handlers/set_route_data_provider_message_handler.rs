use crate::routes::RouteDataProvider;
use app_world::AppWorldWrapper;

use crate::app::World;

impl World {
    pub(super) fn set_route_data_provider_message_handler(&mut self, wrap: AppWorldWrapper<Self>) {
        self.resources.router.provide(RouteDataProvider::new(wrap));
    }
}
