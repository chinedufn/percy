use std::collections::HashMap;
use virtual_dom_rs::View;

type ViewFn = Box<Fn(HashMap<String, String>) -> Box<View>>;
type ParamTypes = HashMap<String, ParamType>;

/// A route specifies a path to match against. When a match is found a `view_creator` is used
/// to return an `impl View` that can be used to render the appropriate content for that route.
pub struct Route {
    route_definition: &'static str,
    param_types: ParamTypes,
    view_creator: ViewFn,
}

/// All of the parameters that our routes can have. This is how we would distinguish "id" in
/// /users/:id
/// from being a u32, u8, or some other value
#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum ParamType {
    U32,
    U64,
}

impl Route {
    /// Create a new Route. You'll usually later call route.match(...) in order to see if a given
    /// the path in the browser URL matches your route's path definition.
    pub fn new(
        route_definition: &'static str,
        param_types: ParamTypes,
        view_creator: ViewFn,
    ) -> Route {
        Route {
            route_definition,
            param_types,
            view_creator,
        }
    }
}

impl Route {
    /// Determine whether or not our route matches a provided path.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// route.matches("/food/tacos");
    /// ```
    pub fn matches(&self, path: &str) -> bool {
        // ex: [ "", "food", ":food_type" ]
        let defined_segments = self
            .route_definition
            .split("/")
            .filter(|segment| segment.len() > 0)
            .collect::<Vec<&str>>();

        // ex: [ "", "food", "tacos" ]
        let incoming_segments = path
            .split("/")
            .filter(|segment| segment.len() > 0)
            .collect::<Vec<&str>>();

        if defined_segments.len() != incoming_segments.len() {
            return false;
        }

        for (index, defined_segment) in defined_segments.iter().enumerate() {
            if defined_segment.len() == 0 {
                continue;
            }

            let mut chars = defined_segment.chars();

            let first_char = chars.next().unwrap();

            // ex: ":food_type"
            if first_char == ':' {
                let param_name = chars.collect::<String>();
                // ex: ParamType::String
                let param_type = self.param_types.get(&param_name).unwrap();

                let incoming_param_value = incoming_segments[index];

                // Make sure that it is possible to convert the String that the user provided
                // into the parameter type that we expect (u32, u8, i8, etc)
                match param_type {
                    ParamType::U32 => {
                        if incoming_param_value.parse::<u32>().is_err() {
                            return false;
                        }
                    }
                    ParamType::U64 => {
                        if incoming_param_value.parse::<u64>().is_err() {
                            return false;
                        }
                    }
                };
            }
        }

        true
    }

    /// Given an incoming path, create the `View` that uses that path data.
    ///
    /// For example.. if our defined path is `/users/:id`
    /// and our incoming path is `/users/5`
    ///
    /// Our view will end up getting created with `id: 5`
    pub fn view(&self, incoming_path: &str) -> Box<View> {
        (self.view_creator)(self.params(incoming_path))
    }

    fn params(&self, incoming_path: &str) -> HashMap<String, String> {
        let incoming_path = incoming_path.split("/").collect::<Vec<&str>>();

        self.route_definition
            .split("/")
            .collect::<Vec<&str>>()
            .iter()
            .enumerate()
            .filter(|(_index, segment)| {
                if segment.len() == 0 {
                    return false;
                }

                segment.chars().next().unwrap() == ':'
            })
            .map(|(index, segment)| (segment.to_string(), incoming_path[index].to_string()))
            .collect::<HashMap<String, String>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use virtual_dom_rs::html;
    use virtual_dom_rs::VirtualNode;

    struct MyView {
        id: u32,
    }

    impl View for MyView {
        fn render(&self) -> VirtualNode {
            let id = VirtualNode::text(self.id.to_string());
            html! { <div> {id} </div> }
        }
    }

    struct MatchRouteTestCase {
        desc: &'static str,
        route_definition: &'static str,
        // (route ... should it match ... description of test)
        matches: Vec<(&'static str, bool, &'static str)>,
    }

    impl MatchRouteTestCase {
        fn test(&self) {
            let view_creator = |params: HashMap<String, String>| {
                Box::new(MyView {
                    id: params.get(":id").unwrap().parse::<u32>().unwrap(),
                }) as Box<View>
            };

            let mut param_types = HashMap::new();
            param_types.insert("id".to_string(), ParamType::U32);

            let route = Route::new(self.route_definition, param_types, Box::new(view_creator));

            for match_case in self.matches.iter() {
                assert_eq!(
                    route.matches(match_case.0),
                    match_case.1,
                    "{}\n{}",
                    self.desc,
                    match_case.2
                );
            }
        }
    }

    #[test]
    fn route_type_safety() {
        MatchRouteTestCase {
            desc: "Typed route parameters",
            route_definition: "/users/:id",
            matches: vec![
                ("/users/5", true, "5 is a u32"),
                ("/users/foo", false, "foo is not a u32"),
            ],
        }
        .test();
    }

    #[test]
    fn route_cascade() {
        MatchRouteTestCase {
            desc: "Make sure that `/` route doesn't capture `/other-routes`",
            route_definition: "/",
            matches: vec![("/foo", false, "routes should not match additional segments")],
        }
        .test();
    }

    #[test]
    fn create_view() {
        assert_eq!(
            create_test_route().view("/users/300").render(),
            html! {<div> 300 </div>},
            "Creates a view from a provided route"
        );
    }

    fn create_test_route() -> Route {
        let view_creator = |params: HashMap<String, String>| {
            Box::new(MyView {
                id: params.get(":id").unwrap().parse::<u32>().unwrap(),
            }) as Box<View>
        };

        let mut param_types = HashMap::new();
        param_types.insert("id".to_string(), ParamType::U32);

        let route = Route::new("/users/:id", param_types, Box::new(view_creator));

        route
    }

    // TODO:
    #[test]
    fn macro_works() {
        //        let route = MyView::route();
        //
        //        assert!(route.matches("/users/5"));
        //        assert!(!route.matches("/users/not_a_u32"));
    }

    struct Store {}

    // TODO: Plan out how to provide a state store to routes on paper. Probably some sort of generic
    // StateStore<T> where T is your applications Store
    //    #[route(path = "/users/:id")]
    struct ViewWithStore {
        id: u32,
        store: Rc<RefCell<Store>>,
    }

    // TODO
    #[test]
    fn provide_state_store() {}
}
