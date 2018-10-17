use std::collections::HashMap;
use std::str::FromStr;
use virtual_dom_rs::View;

type ViewFn = Box<Fn(HashMap<String, String>) -> Box<View>>;

/// A route specifies a path to match against. When a match is found a `view_creator` is used
/// to return an `impl View` that can be used to render the appropriate content for that route.
pub struct Route<'a> {
    path_catcher: &'a str,
    param_types: HashMap<String, ParamType>,
    view_creator: ViewFn,
}

/// All of the parameters that our routes can have. This is how we would distinguish "id" in
/// /users/:id
/// from being a u32, u8, or some other value
pub enum ParamType {
    U32,
}

impl<'a> Route<'a> {
    /// Create a new Route. You'll usually later call route.match(...) in order to see if a given
    /// the path in the browser URL matches your route's path definition.
    pub fn new(path: &str, param_types: HashMap<String, ParamType>, view_creator: ViewFn) -> Route {
        Route {
            path_catcher: path,
            param_types,
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
        // ex: [ "", "food", ":food_type" ]
        let route_segments = self.path_catcher.split("/").collect::<Vec<&str>>();

        // ex: [ "", "food", "tacos" ]
        let user_segments = path.split("/").collect::<Vec<&str>>();

        for (index, segment) in route_segments.iter().enumerate() {
            if segment.len() == 0 {
                continue;
            }

            let mut chars = segment.chars();

            let first_char = chars.next().unwrap();

            // ex: ":food_type"
            if first_char == ':' {
                let param_name = chars.collect::<String>();
                // ex: ParamType::String
                let param_type = self.param_types.get(&param_name).unwrap();

                let user_provided_param = user_segments[index];

                // Make sure that it is possible to convert the String that the user provided
                // into the parameter type that we expect (u32, u8, i8, etc)
                match param_type {
                    ParamType::U32 => {
                        if user_provided_param.parse::<u32>().is_err() {
                            return false;
                        }
                    }
                };
            }
        }

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

        let mut param_types = HashMap::new();
        param_types.insert("id".to_string(), ParamType::U32);

        let route = Route::new("/users/:id", param_types, Box::new(view_creator));

        assert!(route.matches("/users/5"), "5 is a u32");
        assert!(!route.matches("/users/foo"), "'foo' is not a u32");
    }
}
