extern crate isomorphic_app;
extern crate isomorphic_server;

use isomorphic_server::server;
use std::env;
use std::thread::Builder;

fn main() {
    env_logger::init();
    server::serve();
}
