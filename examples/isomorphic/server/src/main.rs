use isomorphic_server::server;

fn main() {
    env_logger::init();
    server::serve();
}
