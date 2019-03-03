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
    routes: Vec<Route>,
}

pub trait RouteHandler {
    fn view(&self, incoming_route: &str) -> VirtualNode;
}

impl Router {
    /// Append a route to our vector of Route's. The order that you add routes matters, as
    /// we'll start from the beginning of the vector when matching routes and return the
    /// first route that matches.
    pub fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    /// Push a vector of Routes into the Router
    pub fn set_routes(&mut self, routes: Vec<Route>) {
        self.routes = routes;
    }

    /// Get the first route in our routes vector view that handles this `incoming_route`
    /// and return the view for that route.
    ///
    /// You'll typically call this when trying to render the correct view based on the
    /// page URL or after clicking on an anchor tag.
    pub fn view(&self, incoming_route: &str) -> Option<VirtualNode> {
        for route in self.routes.iter() {
            if route.matches(incoming_route) {
                return Some(route.view(incoming_route));
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
