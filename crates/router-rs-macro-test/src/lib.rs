#![feature(proc_macro_hygiene)]

use router_rs::prelude::*;
use router_rs_macro::{create_routes, route};
use virtual_node::VirtualNode;

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
    count: u8
}

#[route(path = "/")]
fn route_provided_data(state: Provided<State>) -> VirtualNode {
    VirtualNode::Text(format!("Count: {}", state.count).into())
}

#[test]
fn provided_data() {
    let mut router = Router::default();

    router.provide(State {count: 50});

    router.set_route_handlers(create_routes![route_provided_data]);

    assert_eq!(
        router.view("/").unwrap(),
        VirtualNode::Text("Count: 50".into())
    );
}

// TODO: Compile time error if the route doesn't start with a `/`.
// Test this with a router-rs-macro-ui crate that uses compiletest-rs

// TODO: Compile time error if the route defines segments that the function
// does not have.
// Test this with a router-rs-macro-ui crate that uses compiletest-rs
