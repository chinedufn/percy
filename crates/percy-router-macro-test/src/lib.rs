use percy_router::prelude::*;
use percy_vdom::prelude::*;
use std::str::FromStr;

mod book_example;
mod on_visit;

// No Params

#[route(path = "/")]
fn no_params() -> VirtualNode {
    VirtualNode::Text("hello world".into())
}

#[test]
fn root_path() {
    let mut router = Router::default();

    router.set_route_handlers(create_routes![no_params]);

    assert_eq!(
        router.view("/").unwrap(),
        VirtualNode::Text("hello world".into())
    );
}

// Route With One Param

#[route(path = "/:id")]
fn route_one_param(id: u32) -> VirtualNode {
    VirtualNode::Text(format!("{}", id).into())
}

#[test]
fn one_param() {
    let mut router = Router::default();

    router.set_route_handlers(create_routes![route_one_param]);

    assert_eq!(router.view("/10").unwrap(), VirtualNode::Text("10".into()));
}

// Route With Two Params

#[route(path = "/user/:user_id/buddies/:buddy_id")]
fn route_two_params(user_id: u64, buddy_id: u32) -> VirtualNode {
    VirtualNode::Text(format!("User {}. Buddy {}", user_id, buddy_id).into())
}

#[test]
fn two_params() {
    let mut router = Router::default();

    router.set_route_handlers(create_routes![route_two_params]);

    assert_eq!(
        router.view("/user/50/buddies/90").unwrap(),
        VirtualNode::Text("User 50. Buddy 90".into())
    );
}

// Route with Provided Data

struct State {
    count: u8,
}

#[route(path = "/")]
fn route_provided_data(state: Provided<State>) -> VirtualNode {
    VirtualNode::Text(format!("Count: {}", state.count).into())
}

#[test]
fn provided_data() {
    let mut router = Router::default();

    router.provide(State { count: 50 });

    router.set_route_handlers(create_routes![route_provided_data]);

    assert_eq!(
        router.view("/").unwrap(),
        VirtualNode::Text("Count: 50".into())
    );
}

// Route with Two Provided Data

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
    let mut router = Router::default();

    router.provide(Count { count: 8 });
    router.provide(Money(99));

    router.set_route_handlers(create_routes![route_provided_two_data]);

    assert_eq!(
        router.view("/").unwrap(),
        VirtualNode::Text("Count: 8. Dollars: 99".into())
    );
}

// Route with Provided Data And Param

struct SomeState {
    happy: bool,
}

#[route(path = "/users/:id")]
fn route_param_and_data(id: u16, state: Provided<SomeState>) -> VirtualNode {
    VirtualNode::Text(format!("User: {}. Happy: {}", id, state.happy).into())
}

#[test]
fn provided_param_and_data() {
    let mut router = Router::default();

    router.provide(SomeState { happy: true });

    router.set_route_handlers(create_routes![route_param_and_data]);

    assert_eq!(
        router.view("/users/12345").unwrap(),
        VirtualNode::Text("User: 12345. Happy: true".into())
    );
}

// Route with Provided Data And Param

#[route(path = "/players/:id")]
fn route_data_and_param(state: Provided<SomeState>, id: u32) -> VirtualNode {
    VirtualNode::Text(format!("Player: {}. Happy: {}", id, state.happy).into())
}

#[test]
fn provided_data_and_param() {
    let mut router = Router::default();

    router.provide(SomeState { happy: false });

    router.set_route_handlers(create_routes![route_data_and_param]);

    assert_eq!(
        router.view("/players/998").unwrap(),
        VirtualNode::Text("Player: 998. Happy: false".into())
    );
}

// TODO: Compile time error if the route doesn't start with a `/`.
// Test this with a percy-router-macro-ui crate that uses compiletest-rs

// TODO: Compile time error if the route defines segments that the function
// does not have.
// Test this with a percy-router-macro-ui crate that uses compiletest-rs
