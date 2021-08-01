use percy_dom::prelude::*;
use percy_router::prelude::*;

mod book_example;
mod on_visit;

// **************************************************
// No Params
// **************************************************

#[route(path = "/")]
fn no_params() -> VirtualNode {
    VirtualNode::Text("hello world".into())
}

#[test]
fn root_path() {
    let router = Router::new(create_routes![no_params]);

    assert_eq!(
        router.view("/").unwrap(),
        VirtualNode::Text("hello world".into())
    );
}

// **************************************************
// Route With One Param
// **************************************************

#[route(path = "/:id")]
fn route_one_param(id: u32) -> VirtualNode {
    VirtualNode::Text(format!("{}", id).into())
}

#[test]
fn one_param() {
    let router = Router::new(create_routes![route_one_param]);

    assert_eq!(router.view("/10").unwrap(), VirtualNode::Text("10".into()));
}

// **************************************************
// Route With Two Params
// **************************************************

#[route(path = "/user/:user_id/buddies/:buddy_id")]
fn route_two_params(user_id: u64, buddy_id: u32) -> VirtualNode {
    VirtualNode::Text(format!("User {}. Buddy {}", user_id, buddy_id).into())
}

#[test]
fn two_params() {
    let router = Router::new(create_routes![route_two_params]);

    assert_eq!(
        router.view("/user/50/buddies/90").unwrap(),
        VirtualNode::Text("User 50. Buddy 90".into())
    );
}

// **************************************************
// Route with Provided Data
// **************************************************

struct State {
    count: u8,
}

#[route(path = "/")]
fn route_provided_data(state: Provided<State>) -> VirtualNode {
    VirtualNode::Text(format!("Count: {}", state.count).into())
}

#[test]
fn provided_data() {
    let mut router = Router::new(create_routes![route_provided_data]);

    router.provide(State { count: 50 });

    assert_eq!(
        router.view("/").unwrap(),
        VirtualNode::Text("Count: 50".into())
    );
}

// **************************************************
// Route with Two Provided Data
// **************************************************

struct Count {
    count: u8,
}

struct Money(u64);

#[route(path = "/")]
fn route_provided_two_data(count: Provided<Count>, dollars: Provided<Money>) -> VirtualNode {
    VirtualNode::Text(format!("Count: {}. Dollars: {}", count.count, dollars.0).into())
}

#[test]
fn provided_two_data() {
    let mut router = Router::new(create_routes![route_provided_two_data]);

    router.provide(Count { count: 8 });
    router.provide(Money(99));

    assert_eq!(
        router.view("/").unwrap(),
        VirtualNode::Text("Count: 8. Dollars: 99".into())
    );
}

// **************************************************
// Route with Provided Data And Param
// **************************************************

struct SomeState {
    happy: bool,
}

#[route(path = "/users/:id")]
fn route_param_and_data(id: u16, state: Provided<SomeState>) -> VirtualNode {
    VirtualNode::Text(format!("User: {}. Happy: {}", id, state.happy).into())
}

#[test]
fn provided_param_and_data() {
    let mut router = Router::new(create_routes![route_param_and_data]);

    router.provide(SomeState { happy: true });

    assert_eq!(
        router.view("/users/12345").unwrap(),
        VirtualNode::Text("User: 12345. Happy: true".into())
    );
}

// **************************************************
// Route with Provided Data And Param
// **************************************************

#[route(path = "/players/:id")]
fn route_data_and_param(state: Provided<SomeState>, id: u32) -> VirtualNode {
    VirtualNode::Text(format!("Player: {}. Happy: {}", id, state.happy).into())
}

#[test]
fn provided_data_and_param() {
    let mut router = Router::new(create_routes![route_data_and_param]);

    router.provide(SomeState { happy: false });

    assert_eq!(
        router.view("/players/998").unwrap(),
        VirtualNode::Text("Player: 998. Happy: false".into())
    );
}

// **************************************************
// Create route in another module
// **************************************************
mod some_module {
    use super::*;

    #[route(path = "/")]
    pub fn route_in_a_module() -> VirtualNode {
        unimplemented!()
    }
}

/// Verify that we can create a route using a path to a function in another module.
#[test]
fn create_route_in_another_module() {
    create_routes![some_module::route_in_a_module];
}

// **************************************************
// No warnings
// **************************************************

/// Verify that the route macro does not generate any warnings.
mod test_no_warnings {
    #![deny(warnings)]

    use super::*;

    #[route(path = "/")]
    fn no_warnings() -> VirtualNode {
        unimplemented!()
    }
}

// TODO: Compile time error if the route doesn't start with a `/`.
// Test this with a ui test using trybuild

// TODO: Compile time error if the route defines segments that the function
// does not have.
// Test this with a ui test using trybuild

// TODO: Test that verifies that we can add route parameters as function arguments in any order.
