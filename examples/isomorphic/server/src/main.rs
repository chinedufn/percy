use isomorphic_server::rocket_server::rocket;

#[cfg(feature = "with-actix")]
use isomorphic_server::actix_server::serve;

fn main() {
    env_logger::init();

    // cargo run -p server-www --features with-rocket
    #[cfg(feature = "with-rocket")]
    rocket().launch();

    // cargo run -p server-www --features with-actix
    #[cfg(feature = "with-actix")]
    serve()
}
