use std::env;

use rocket::http::ContentType;
use rocket::response::Response;
use rocket::{Rocket, Config};

use isomorphic_app::App;
use std::io::Cursor;
use rocket::config::Environment;
use rocket_contrib::serve::StaticFiles;

const HTML_PLACEHOLDER: &str = "#HTML_INSERTED_HERE_BY_SERVER#";
const STATE_PLACEHOLDER: &str = "#INITIAL_STATE_JSON#";

static INDEX_HTML: &str = include_str!("./index.html");

/// Create a Rocket server for our application
pub fn rocket() -> Rocket {
    let config = Config::build(Environment::Development)
        .address("127.0.0.1")
        .port(7878)
        .unwrap();

    let static_files = format!("{}/../client/build", env!("CARGO_MANIFEST_DIR"));

    rocket::custom(config)
        .mount("/", routes![index, favicon, catch_all])
        .mount("/static", StaticFiles::from(static_files.as_str()))
}

/// # Example
///
/// localhost:7878/?init=50
#[get("/?<initial_count>")]
fn index(initial_count: Option<u32>) -> Result<Response<'static>, ()> {
    respond("/".to_string(), initial_count)
}

/// # Example
///
/// localhost:7878/contributors?init=1200
#[get("/<path>?<initial_count>")]
fn catch_all(path: String, initial_count: Option<u32>) -> Result<Response<'static>, ()> {
    respond(path, initial_count)
}

#[get("/favicon.ico")]
fn favicon() -> &'static str {
    ""
}

fn respond(path: String, initial_count: Option<u32>) -> Result<Response<'static>, ()> {
    let app = App::new(
        initial_count.unwrap_or(1000),
        path,
    );
    let state = app.store.borrow();

    let html = format!("{}", include_str!("./index.html"));
    let html = html.replacen(HTML_PLACEHOLDER, &app.render().to_string(), 1);
    let html = html.replacen(STATE_PLACEHOLDER, &state.to_json(), 1);

    let mut response = Response::new();
    response.set_header(ContentType::HTML);
    response.set_sized_body(Cursor::new(html));

    Ok(response)
}
