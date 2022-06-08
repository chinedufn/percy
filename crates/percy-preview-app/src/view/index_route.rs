use crate::routes::RouteDataProvider;
use percy_dom::prelude::*;
use percy_router::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

impl RouteDataProvider {
    fn get_index_route_data(&self) -> IndexRouteData {
        let world = self.world.read();

        let previews: Vec<PreviewListEntry> = world
            .previews
            .iter()
            .map(|preview| PreviewListEntry {
                name: preview.name().to_string(),
                name_url_friendly: preview.name_url_friendly().to_string(),
            })
            .collect();

        // TODO: Choose based on the current path instead of just grabbing the first component
        let active_preview = if previews.len() > 0 {
            Some(world.previews[0].render().clone())
        } else {
            None
        };

        IndexRouteData {
            preview_list: previews,
            active_preview,
        }
    }
}

struct IndexRouteData {
    preview_list: Vec<PreviewListEntry>,
    active_preview: Option<Rc<RefCell<dyn FnMut() -> VirtualNode>>>,
}

struct PreviewListEntry {
    name: String,
    name_url_friendly: String,
}

#[route(path = "/")]
pub(crate) fn render_index_route(provider: Provided<RouteDataProvider>) -> VirtualNode {
    let route_data = provider.get_index_route_data();
    IndexView { data: route_data }.render()
}

struct IndexView {
    data: IndexRouteData,
}

impl View for IndexView {
    fn render(&self) -> VirtualNode {
        let preview_list = self.render_preview_list();
        let active_preview = self.render_active_preview();

        html! {
            <div
                class=css!("display-flex")
            >
                {preview_list}
                {active_preview}
            </div>
        }
    }
}

impl IndexView {
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
                <h2>Component Previews</h2>
                {preview_list}
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
