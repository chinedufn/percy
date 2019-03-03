//! Powers routing for frontend web applications

use crate::Route;
use virtual_dom_rs::prelude::*;

/// Holds all of the routes for an application.
///
/// A typical use case is that when we want to move to a new route
/// (such as after clicking on an anchor tag)
/// we'll query our router to see if the new route matches any of our route definitions.
///
/// Then if we find a matching route we'll return it.
#[derive(Default)]
pub struct Router {
    route_handlers: Vec<Box<dyn RouteHandler>>,
}

/// Used by router-rs-macro during code generation when turning your
///
/// #[route(path="/...")] macro into a struct.
pub trait RouteHandler {
    fn route(&self) -> &Route;

    fn view(&self, incoming_route: &str) -> VirtualNode;

    fn matches(&self, incoming_path: &str) -> bool {
        self.route().matches(incoming_path)
    }
}

impl Router {
    /// Push a vector of Routes into the Router
    pub fn set_route_handlers(&mut self, route_handlers: Vec<Box<dyn RouteHandler>>) {
        self.route_handlers = route_handlers;
    }

    /// Get the first route in our routes vector view that handles this `incoming_route`
    /// and return the view for that route.
    ///
    /// You'll typically call this when trying to render the correct view based on the
    /// page URL or after clicking on an anchor tag.
    pub fn view(&self, incoming_route: &str) -> Option<VirtualNode> {
        for route_handler in self.route_handlers.iter() {
            if route_handler.matches(incoming_route) {
                return Some(route_handler.view(incoming_route));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foo() {
        // TODO: Add some routes and then make sure that `router.view` works
    }
}
