use crate::routes::RouteDataProvider;
use percy_dom::prelude::*;
use percy_router::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// A page that lets you view a component preview.
#[route(path = "/components/:component_name_url_friendly")]
pub(crate) fn render_visualize_component_route(
    provider: Provided<RouteDataProvider>,
    component_name_url_friendly: String,
) -> VirtualNode {
    let route_data = provider.get_visualize_component_route_data(&component_name_url_friendly);
    ComponentView { data: route_data }.render()
}

impl RouteDataProvider {
    fn get_visualize_component_route_data(
        &self,
        component_name_url_friendly: &str,
    ) -> ComponentVisualizerRouteData {
        let world = self.world.read();

        let preview_list: Vec<PreviewListEntry> = world
            .previews
            .iter()
            .map(|preview| PreviewListEntry {
                name: preview.name().to_string(),
                name_url_friendly: preview.name_url_friendly().to_string(),
            })
            .collect();

        let active_preview = world.previews.iter().find_map(|preview| {
            if preview.name_url_friendly() != component_name_url_friendly {
                return None;
            }

            Some(preview.renderer().clone())
        });

        ComponentVisualizerRouteData {
            preview_list,
            active_preview,
        }
    }
}

struct ComponentVisualizerRouteData {
    preview_list: Vec<PreviewListEntry>,
    active_preview: Option<Rc<RefCell<dyn FnMut() -> VirtualNode>>>,
}

struct PreviewListEntry {
    name: String,
    name_url_friendly: String,
}

struct ComponentView {
    data: ComponentVisualizerRouteData,
}

impl View for ComponentView {
    fn render(&self) -> VirtualNode {
        let preview_list = self.render_preview_list();
        let active_preview = self.render_active_preview();

        // TODO: Create a `MainContentView` that renders the main navigation and some content.
        //  Re-use this across the index route and the visualize component route
        html! {
            <div
                class=css!("display-flex")
            >
                {preview_list}
                <div>
                    {active_preview}
                </div>
            </div>
        }
    }
}

impl ComponentView {
    /// Render a list of links to component previews.
    /// Clicking on a link will show the relevant component.
    fn render_preview_list(&self) -> VirtualNode {
        let preview_list: Vec<VirtualNode> = self
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

    /// Render the component that is currently being previewed.
    fn render_active_preview(&self) -> Option<VirtualNode> {
        let data = &self.data;

        let active_preview = data.active_preview.as_ref()?;
        let active_preview = (active_preview.borrow_mut())();

        let active_preview = html! {
            <div>
                {active_preview}
            </div>
        };

        Some(active_preview)
    }
}
