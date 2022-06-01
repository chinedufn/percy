use crate::app::World;
use app_world::AppWorldWrapper;
use percy_router::prelude::*;

pub(crate) fn init_router(route_data_provider: RouteDataProvider) {
    use crate::view;

    let mut router = Router::new(create_routes![]);
    router.provide(route_data_provider)
}

/// Used to
pub(crate) struct RouteDataProvider {
    world: AppWorldWrapper<World>,
}
