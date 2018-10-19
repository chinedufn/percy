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

impl Router {
    /// Append a route to our vector of Route's. The order that you add routes matters, as
    /// we'll start from the beginning of the vector when matching routes and return the
    /// first route that matches.
    pub fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    /// Get the first route in our routes vector view that handles this `incoming_route`
    /// and return the view for that route.
    ///
    /// You'll typically call this when trying to render the correct view based on the
    /// page URL or after clicking on an anchor tag.
    pub fn view(&self, incoming_route: &str) -> Option<Box<View>> {
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
    use crate::route::ParamType;
    use std::collections::HashMap;
    use virtual_dom_rs::html;
    use virtual_dom_rs::recurse_html;

    struct TestView {
        kind: &'static str,
    }

    impl View for TestView {
        fn render(&self) -> VirtualNode {
            html! {<div> {self.kind} </div> }
        }
    }

    #[test]
    fn match_route() {
        let mut router = Router::default();

        let mut param_types = HashMap::new();
        param_types.insert("id".to_string(), ParamType::U64);

        let view_creator = Box::new(|_| Box::new(TestView { kind: "first" }) as Box<View>);
        let first_route = Route::new("/users/:id", param_types, view_creator);

        let mut param_types = HashMap::new();
        param_types.insert("id".to_string(), ParamType::U64);

        let view_creator = Box::new(|_| Box::new(TestView { kind: "second" }) as Box<View>);
        let second_route = Route::new("/users/:id/name", param_types, view_creator);

        router.add_route(first_route);
        router.add_route(second_route);

        assert_eq!(
            router.view("/users/5/name").unwrap().render(),
            html! { <div> {"second"} </div>}
        );
    }
}
