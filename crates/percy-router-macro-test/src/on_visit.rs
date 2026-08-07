use percy_dom::prelude::VirtualNode;
use percy_dom::VirtualNodeWebSys;
use percy_router::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicUsize};

static mut VISITED: AtomicBool = AtomicBool::new(false);

// ==================================================
// on_visit with no parameters
// ==================================================

#[route(
    path = "/",
    on_visit = set_visited_true
)]
fn on_visit_works() -> VirtualNodeWebSys {
    VirtualNode::Text("On Visit".into())
}

fn set_visited_true() {
    unsafe {
        *VISITED.get_mut() = true;
    }
}

#[test]
fn visit() {
    use std::sync::atomic::Ordering;

    let router = Router::new(create_routes![on_visit_works]);

    unsafe {
        assert_eq!(VISITED.load(Ordering::SeqCst), false);
    }

    router.matching_route_handler("/").unwrap().on_visit("/");

    unsafe {
        assert_eq!(VISITED.load(Ordering::SeqCst), true);
    }
}

// ==================================================
// on_visit with parameters provided data
// ==================================================

static mut ID: AtomicUsize = AtomicUsize::new(0);

struct SomeState {
    #[allow(unused)]
    happy: bool,
}

#[route(
  path = "/users/:id",
  on_visit = set_id
)]
#[allow(unused)]
fn route_param_and_data(id: u16, state: Provided<SomeState>) -> VirtualNodeWebSys {
    VirtualNode::Text("".into())
}

#[allow(unused)]
fn set_id(id: u16, state: Provided<SomeState>) {
    unsafe {
        *ID.get_mut() = id as usize;
    }
}

#[test]
fn visit_params_data() {
    use std::sync::atomic::Ordering;

    let mut router = Router::new(create_routes![route_param_and_data]);
    router.provide(SomeState { happy: false });

    unsafe {
        assert_eq!(ID.load(Ordering::SeqCst), 0);
    }

    router
        .matching_route_handler("/users/5")
        .unwrap()
        .on_visit("/users/5");

    unsafe {
        assert_eq!(ID.load(Ordering::SeqCst), 5);
    }
}
