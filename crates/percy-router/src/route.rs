use std::fmt::Debug;
use std::fmt::Error;
use std::fmt::Formatter;
use std::str::FromStr;

/// Enables a type to be used as a route paramer
///
/// ```ignore
/// // Example of a route param that only matches id's that are less than
/// // 10 characters long
///
/// #[route(path = "/path/to/:id")
/// fn my_route (id: ShortId) -> VirtualNode {
/// }
///
/// struct ShortId {
///     id: String,
///     length: usize
/// }
///
/// impl RouteParam for MyCustomType {
///     fn from_str (param: &str) -> Result<MyCustomType, ()> {
///         if param.len() > 10 {
///             Ok(MyCustomType {
///                 length: param.len(), id: param.to_string()
///             })
///         } else {
///             Err(())
///         }
///     }
/// }
/// ```
pub trait RouteParam {
    /// Given some parameter, return Self
    ///
    /// For example, for the route path:
    ///
    ///   /route/path/:id
    ///
    /// And incoming path
    ///
    ///   /route/path/55
    ///
    /// If Self is a u32 we would return 55
    fn from_str_param(param: &str) -> Result<Self, &str>
    where
        Self: Sized;
}

impl<T> RouteParam for T
where
    T: FromStr,
{
    fn from_str_param(param: &str) -> Result<T, &str> {
        match param.parse::<T>() {
            Ok(parsed) => Ok(parsed),
            Err(_) => Err(param),
        }
    }
}

/// Given a param_key &str and a param_val &str, get the corresponding route parameter
///
/// ex: ("friend_count", "30")
pub type ParseRouteParam = Box<dyn Fn(&str, &str) -> Option<Box<dyn RouteParam>>>;

/// A route specifies a path to match against. When a match is found a `view_creator` is used
/// to return an `impl View` that can be used to render the appropriate content for that route.
pub struct Route {
    route_definition: &'static str,
    // FIXME: Do we need this to return the RouteParam ... or do we really just need a bool
    // to check if the route exists? Seems like we're only using the boolean
    route_param_parser: ParseRouteParam,
}

impl Debug for Route {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str(self.route_definition)?;
        Ok(())
    }
}

impl Route {
    /// Create a new Route. You'll usually later call route.match(...) in order to see if a given
    /// the path in the browser URL matches your route's path definition.
    pub fn new(route_definition: &'static str, route_param_parser: ParseRouteParam) -> Route {
        Route {
            route_definition,
            route_param_parser,
        }
    }
}

impl Route {
    /// Determine whether or not our route matches a provided path.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // path = "/food/:food_type"
    ///
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

        // If we defined a certain number of segments and we don't see the same amount in
        // the incoming route, we know that it isn't a match
        if defined_segments.len() != incoming_segments.len() {
            return false;
        }

        // Iterate over every segment and verify that they match, or if it's a
        // RouteParam segment verify that we can parse it
        for (index, defined_segment) in defined_segments.iter().enumerate() {
            if defined_segment.len() == 0 {
                continue;
            }

            let mut chars = defined_segment.chars();

            let first_char = chars.next().unwrap();

            // ex: ":food_type"
            if first_char == ':' {
                // food_type
                let param_name = chars.collect::<String>();

                // tacos
                let incoming_param_value = incoming_segments[index];

                let matches =
                    (self.route_param_parser)(param_name.as_str(), incoming_param_value).is_some();
                if !matches {
                    return false;
                }
            } else {
                // Compare segments on the same level
                let incoming_segment = incoming_segments[index];

                if defined_segment != &incoming_segment {
                    return false;
                }
            }
        }

        true
    }

    /// Given an incoming path and a param_key, get the RouteParam
    pub fn find_route_param<'a>(&self, incoming_path: &'a str, param_key: &str) -> Option<&'a str> {
        let param_key = format!(":{}", param_key);

        let mut incoming_segments = incoming_path.split("/");

        for (idx, defined_segment) in self.route_definition.split("/").enumerate() {
            if defined_segment == &param_key {
                for _ in 0..idx {
                    incoming_segments.next().unwrap();
                }
                return Some(incoming_segments.next().unwrap());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that we can get the route parameter value for a given route parameter.
    ///
    /// For a route definition is `/:id`, and a path "/5" we confirm that when we ask for the
    /// "id" parameter of the path we get back "5".
    #[test]
    fn find_route_param() {
        let route = Route::new(
            "/:id",
            Box::new(|param_key, param_val| {
                if param_key == "id" {
                    Some(Box::new(u32::from_str_param(param_val).unwrap()));
                }

                None
            }),
        );

        assert_eq!(route.find_route_param("/5", "id"), Some("5"));
    }

    /// Verify that route parameters such as `/:id` are typed.
    ///
    /// For instance, confirm that a route parameter that has an integer type does not match a
    /// string.
    #[test]
    fn route_type_safety() {
        MatchRouteTestCase {
            route_definition: "/users/:id",
            matches: vec![
                // Verify that an integer matches a parameter that is defined as an integer.
                ("/users/5", true),
                // Verify that a string does not match a parameter that is defined as an integer.
                ("/users/foo", false),
            ],
        }
            .test();
    }

    /// Verify that a `/` route definition doesn't capture `/some-route`.
    #[test]
    fn route_cascade() {
        MatchRouteTestCase {
            route_definition: "/",
            matches: vec![
                // Verify that a "/" segment is not a catch-all.
                // So, "/" should not match "/foo".
                ("/foo", false),
            ],
        }
            .test();
    }

    /// Verify that we correctly match when a static segment comes after a dynamic segment.
    ///
    /// This helps ensure that are not ignoring segments that come after a dynamic segment.
    #[test]
    fn static_segment_after_dynamic_segment() {
        MatchRouteTestCase {
            route_definition: "/:id/foo",
            matches: vec![
                // Verify that a correct segment after a dynamic segment leads to a match.
                ("/5/foo", true),
                // Verify that an incorrect segment after a dynamic segment does not lead to a match.
                ("/5/bar", false),
            ],
        }
            .test();
    }

    struct MatchRouteTestCase {
        route_definition: &'static str,
        // (path, should it match)
        matches: Vec<(&'static str, bool)>,
    }

    impl MatchRouteTestCase {
        fn test(&self) {
            fn get_param(param_key: &str, param_val: &str) -> Option<Box<dyn RouteParam>> {
                // /some/route/path/:id/
                match param_key {
                    "id" => match u32::from_str_param(param_val) {
                        Ok(num) => Some(Box::new(num)),
                        Err(_) => None,
                    },
                    _ => None,
                }
            }

            let route = Route::new(self.route_definition, Box::new(get_param));

            for match_case in self.matches.iter() {
                assert_eq!(
                    route.matches(match_case.0),
                    match_case.1,
                    "Test case failed: {}",
                    match_case.0,
                );
            }
        }
    }
}
