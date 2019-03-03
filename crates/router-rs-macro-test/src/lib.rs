#![feature(proc_macro_hygiene)]

use router_rs::prelude::*;
use router_rs_macro::{create_routes, route};
use virtual_node::VText;
use virtual_node::VirtualNode;

// No Params

#[route(path = "/")]
fn no_params() -> VirtualNode {
    VirtualNode::Text(VText::new("hello world"))
}

#[test]
fn root_path() {
    let mut router = Router::default();

    router.set_route_handlers(create_routes![no_params]);

    assert_eq!(
        router.view("/").unwrap(),
        VirtualNode::Text(VText::new("hello world"))
    );
}

// Route With One Param

#[route(path = "/:id")]
fn route_one_param(id: u32) -> VirtualNode {
    VirtualNode::Text(VText::new(format!("{}", id).as_str()))
}

#[test]
fn one_param() {
    let mut router = Router::default();

    router.set_route_handlers(create_routes![route_one_param]);

    assert_eq!(
        router.view("/10").unwrap(),
        VirtualNode::Text(VText::new("10"))
    );
}
