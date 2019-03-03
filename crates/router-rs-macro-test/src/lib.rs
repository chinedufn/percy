#![feature(proc_macro_hygiene)]

use router_rs::prelude::*;
use router_rs_macro::{create_routes, route};
use std::vec::IntoIter;
use virtual_node::VText;
use virtual_node::VirtualNode;

#[test]
fn root_path() {
    #[route(path = "/")]
    fn root_route() -> VirtualNode {
        VirtualNode::Text(VText::new("hello world"))
    }

    let mut router = Router::default();

    router.set_routes(create_routes![root_route]);

    assert_eq!(
        router.view("/").unwrap(),
        VirtualNode::Text(VText::new("hello world"))
    );
}

//pub mod __root_route_module__ {
//
//    #[allow(non_camel_case_types)]
//
//    pub struct root_route {
//        pub route: router_rs::Route,
//    }
//}

