//! Powers routing for frontend web applications

use crate::Route;
use percy_dom::prelude::*;
use std::any::Any;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A map of TypeId's to Box<Provided<T>> (stored as Box<dyn Any>)
pub type ProvidedMap = Rc<RefCell<HashMap<TypeId, Box<dyn Any>>>>;

/// Holds all of the routes for an application.
///
/// A typical use case is that when we want to move to a new route
/// (such as after clicking on an anchor tag)
/// we'll query our router to see if the new route matches any of our route definitions.
///
/// Then if we find a matching route we'll return it.
pub struct Router {
    route_handlers: Vec<Rc<dyn RouteHandler>>,
    pub(crate) provided: ProvidedMap,
}

// Used by percy-router-macro during code generation when turning your
//
// #[route(path="/...")] macro into a struct.
#[doc(hidden)]
pub trait RouteHandler {
    fn route(&self) -> &Route;

    fn view(&self, incoming_route: &str) -> VirtualNode;

    fn set_provided(&self, provided: ProvidedMap);

    fn provided(&self) -> std::cell::Ref<'_, ProvidedMap>;

    fn matches(&self, incoming_path: &str) -> bool {
        self.route().matches(incoming_path)
    }

    /// What to do when this route is visited.
    ///
    /// ex: make an HTTP request to load some data for application state.
    fn on_visit(&self, incoming_path: &str);
}

impl Router {
    /// Create a new Router.
    ///
    /// ```no_run,ignore
    /// # use percy_router::prelude::Router;
    /// let router = Router::new(create_routes![index_route, products_route]);
    /// ```
    pub fn new(mut route_handlers: Vec<Rc<dyn RouteHandler>>) -> Self {
        let provided = Rc::new(RefCell::new(HashMap::new()));

        for route_handler in route_handlers.iter_mut() {
            route_handler.set_provided(Rc::clone(&provided));
        }

        Self {
            route_handlers,
            provided,
        }
    }

    /// Return the matching RouteHandler given some `incoming_route`
    pub fn matching_route_handler(&self, incoming_route: &str) -> Option<&Rc<dyn RouteHandler>> {
        for route_handler in self.route_handlers.iter() {
            if route_handler.matches(incoming_route) {
                return Some(route_handler);
            }
        }

        None
    }

    /// Get the first route in our routes vector view that handles this `incoming_route`
    /// and return the view for that route.
    ///
    /// You'll typically call this when trying to render the correct view based on the
    /// page URL or after clicking on an anchor tag.
    pub fn view(&self, incoming_route: &str) -> Option<VirtualNode> {
        match self.matching_route_handler(incoming_route) {
            Some(route_handler) => Some(route_handler.view(incoming_route)),
            None => None,
        }
    }
}
