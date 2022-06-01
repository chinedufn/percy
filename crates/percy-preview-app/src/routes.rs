use crate::app::World;
use app_world::AppWorldWrapper;
use percy_router::prelude::*;

pub(crate) fn create_router() -> Router {
    use crate::view;

    let router = Router::new(create_routes![view::index_route::render_index_route]);

    router
}

/// Used to
pub(crate) struct RouteDataProvider {
    world: AppWorldWrapper<World>,
}

impl RouteDataProvider {
    /// Create a new RouteDataProvider
    pub fn new(world: AppWorldWrapper<World>) -> RouteDataProvider {
        RouteDataProvider { world }
    }
}
