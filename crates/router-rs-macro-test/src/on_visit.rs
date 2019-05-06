use router_rs::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use virtual_dom_rs::prelude::VirtualNode;

static mut VISITED: AtomicBool = AtomicBool::new(false);

#[route(
    path = "/",
    on_visit = set_visited_true
)]
fn on_visit_works() -> VirtualNode {
    VirtualNode::Text("On Visit".into())
}

fn set_visited_true() {
    unsafe {
        *VISITED.get_mut() = true;
    }
}

#[test]
fn visit() {
    let mut router = Router::default();

    router.set_route_handlers(create_routes![on_visit_works]);

    unsafe {
        assert_eq!(VISITED.load(Ordering::SeqCst), false);
    }

    router.matching_routerhandler("/").unwrap().on_visit();

    unsafe {
        assert_eq!(VISITED.load(Ordering::SeqCst), true);
    }
}
