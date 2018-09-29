use std::collections::HashMap;
use virtual_dom_rs::View;

type ViewFn = Box<Fn(HashMap<String, String>) -> Box<View>>;

/// A route specifies a path to match against. When a match is found a `view_creator` is used
/// to return an `impl View` that can be used to render the appropriate content for that route.
pub struct Route<'a> {
    path_matcher: &'a str,
    view_creator: ViewFn,
}

impl<'a> Route<'a> {
    /// Create a new Route. You'll usually later call route.match(...) in order to see if a given
    /// the path in the browser URL matches your route's path definition.
    pub fn new(path: &str, view_creator: ViewFn) -> Route {
        Route {
            path_matcher: path,
            view_creator,
        }
    }
}

impl<'a> Route<'a> {
    /// Determine whether or not our route matches a provided path.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// route.matches("/food/tacos");
    /// ```
    pub fn matches(&self, path: &str) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use virtual_dom_rs::virtual_node::VirtualNode;
    use virtual_dom_rs::{html, recurse_html};

    #[test]
    fn match_route() {
        struct MyView {
            id: u32,
        }

        impl View for MyView {
            fn render(&self) -> VirtualNode {
                html! { <div> </div> }
            }
        }

        let view_creator = |params: HashMap<String, String>| {
            Box::new(MyView {
                id: params.get("id").unwrap().parse::<u32>().unwrap(),
            }) as Box<View>
        };

        let route = Route::new("/users/:id", Box::new(view_creator));

        assert!(route.matches("/users/5"), "5 is a u32");
        assert!(!route.matches("/users/foo"), "'foo' is not a u32");
    }
}
