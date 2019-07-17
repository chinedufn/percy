#[cfg(feature = "with-rocket")]
use isomorphic_server::rocket_server::rocket;

#[cfg(feature = "with-actix")]
use isomorphic_server::actix_server::serve;

fn main() {
    env_logger::init();

    let static_files = {
        // Development
        #[cfg(debug_assertions)]
        {
            format!("{}/../client/build", env!("CARGO_MANIFEST_DIR"))
        }

        // Production
        #[cfg(not(debug_assertions))]
        {
            format!("{}/../client/dist", env!("CARGO_MANIFEST_DIR"))
        }
    };

    // cargo run -p server-www --features with-rocket
    #[cfg(feature = "with-rocket")]
    rocket(static_files).launch();

    // cargo run -p server-www --features with-actix
    #[cfg(feature = "with-actix")]
    serve(static_files)
}
