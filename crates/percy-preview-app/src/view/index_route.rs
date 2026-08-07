use crate::routes::RouteDataProvider;
use percy_dom::prelude::*;
use percy_dom::VirtualNodeWebSys;
use percy_router::prelude::*;

#[route(path = "/")]
pub(crate) fn render_index_route(provider: Provided<RouteDataProvider>) -> VirtualNodeWebSys {
    let route_data = provider.get_index_route_data();
    IndexView { data: route_data }.render()
}

impl RouteDataProvider {
    fn get_index_route_data(&self) -> IndexRouteData {
        let world = self.world.read();

        let preview_list: Vec<PreviewListEntry> = world
            .previews
            .iter()
            .map(|preview| PreviewListEntry {
                name: preview.name().to_string(),
                name_url_friendly: preview.name_url_friendly().to_string(),
            })
            .collect();

        IndexRouteData { preview_list }
    }
}

struct IndexRouteData {
    preview_list: Vec<PreviewListEntry>,
}

struct PreviewListEntry {
    name: String,
    name_url_friendly: String,
}

struct IndexView {
    data: IndexRouteData,
}

impl View<web_sys::Window> for IndexView {
    fn render(&self) -> VirtualNodeWebSys {
        let preview_list = self.render_preview_list();

        // TODO: Create a `MainContentView` that renders the main navigation and some content.
        //  Re-use this across the index route and the visualize component route
        html! {
            <div
                class=css!("display-flex")
            >
              {preview_list}
              <div>
                Home page here
              </div>
            </div>
        }
    }
}

impl IndexView {
    /// Render a list of links to component previews.
    /// Clicking on a link will show the relevant component.
    fn render_preview_list(&self) -> VirtualNodeWebSys {
        let preview_list: Vec<VirtualNodeWebSys> = self
            .data
            .preview_list
            .iter()
            .map(|p| {
                let link = format!("/components/{}", p.name_url_friendly);
                let name = &p.name;

                html! {
                    <div>
                        <a href=link>{ name }</a>
                    </div>
                }
            })
            .collect();

        html! {
            <div>
                <div class=css!("mb10")>
                  <a href="/">Home</a>
                </div>

                <div>
                    <h3>Components</h3>
                    {preview_list}
                </div>
            </div>
        }
    }
}
