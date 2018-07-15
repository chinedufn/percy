extern crate isomorphic;
extern crate virtual_dom_rs;
use virtual_dom_rs::VirtualNode;

use isomorphic::server;

fn main() {
    let html = VirtualNode::new("div");

    server::serve();
}
