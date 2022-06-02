use crate::routes::RouteDataProvider;
use percy_dom::prelude::*;
use percy_router::prelude::*;

impl RouteDataProvider {
    fn get_index_route_data(&self) -> IndexRouteData {
        IndexRouteData {}
    }
}

struct IndexRouteData {
    //
}

#[route(path = "/")]
pub(crate) fn render_index_route(provider: Provided<RouteDataProvider>) -> VirtualNode {
    let route_data = provider.get_index_route_data();

    html! {
        <div>Percy preview app here</div>
    }
}
