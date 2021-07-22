#![recursion_limit = "128"]
#[deny(missing_docs)]
extern crate proc_macro;

mod create_routes_macro;
mod route_macro;

/// An attribute that turns a function into a view route
///
/// ```ignore
/// #[route(path = "/users/:user_id")]
/// fn my_route(user_id: u32) -> VirtualNode {
///     let user_id = format!("{}", user_id);
///     html! { <div id=user_id> World </div> }
/// }
///
/// fn main() {
///     let mut router = Router::new(create_routes![
///         my_route,
///     ]);
///
///     assert_eq!(
///         router.view("/users/5").unwrap(),
///         html! { <div id="5"> Hello World </div> }
///     );
///
/// }
/// ```
#[proc_macro_attribute]
pub fn route(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    route_macro::route(args, input)
}

/// ```ignore
/// #[route(path = "/")]
/// fn my_route() -> VirtualNode {
///     html! { Hello World }
/// }
///
/// #[route(path = "/:id")]
/// fn route2(id: u8) -> VirtualNode {
///     html! { Route number 2 }
/// }
///
/// fn main() {
///     let mut router = Router::new(create_routes![
///         my_route,
///         route2
///     ]);
///
///     assert_eq!(
///         router.view("/").unwrap(),
///         html! { Hello World }
///     );
///
/// }
/// ```
#[proc_macro]
pub fn create_routes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    create_routes_macro::create_routes(input)
}
