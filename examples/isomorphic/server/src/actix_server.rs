use actix_files as fs;
use actix_rt::System;
use actix_web::{
    web, App as ActixApp, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder,
};
use serde::Deserialize;

use isomorphic_app::App;

const HTML_PLACEHOLDER: &str = "#HTML_INSERTED_HERE_BY_SERVER#";
const STATE_PLACEHOLDER: &str = "#INITIAL_STATE_JSON#";

#[derive(Deserialize, Debug)]
struct RequestQuery {
    init: Option<u32>,
}

/// # Example
///
/// localhost:7878/?init=50
fn index(query: web::Query<RequestQuery>) -> impl Responder {
    respond("/".to_string(), query.init)
}

/// # Example
///
/// localhost:7878/contributors?init=1200
fn catch_all(req: HttpRequest) -> impl Responder {
    let query = web::Query::<RequestQuery>::extract(&req).unwrap();
    let path = web::Path::<String>::extract(&req).unwrap();
    respond(path.to_string(), query.init)
}

fn respond(path: String, init: Option<u32>) -> impl Responder {
    let app = App::new(init.unwrap_or(1001), path);
    let state = app.store.borrow();

    let html = format!("{}", include_str!("./index.html"));
    let html = html.replacen(HTML_PLACEHOLDER, &app.render().to_string(), 1);
    let html = html.replacen(STATE_PLACEHOLDER, &state.to_json(), 1);
    HttpResponse::Ok().content_type("text/html").body(html)
}

pub fn serve(static_files: String) {
    let sys = System::new("actix-isomorphic");

    HttpServer::new(move || {
        ActixApp::new()
            .route("/", web::get().to(index))
            .route("/{path}", web::get().to(catch_all))
            .service(fs::Files::new("/static", static_files.as_str()).show_files_listing())
    })
    .bind("0.0.0.0:7878")
    .unwrap()
    .start();

    println!("Actix server listening on port 7878");

    let _ = sys.run();
}
