use crate::app::World;
use app_world::AppWorldWrapper;
use percy_router::prelude::*;

pub(crate) fn create_router() -> Router {
    use crate::view;

    let router = Router::new(create_routes![
        view::index_route::render_index_route,
        view::visualize_component_route::render_visualize_component_route,
    ]);

    router
}

/// Used to provide data to a route.
pub(crate) struct RouteDataProvider {
    pub(crate) world: AppWorldWrapper<World>,
}

impl RouteDataProvider {
    /// Create a new RouteDataProvider
    pub fn new(world: AppWorldWrapper<World>) -> RouteDataProvider {
        RouteDataProvider { world }
    }
}
