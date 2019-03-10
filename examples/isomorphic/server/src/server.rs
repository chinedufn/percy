extern crate actix_web;
use self::actix_web::{fs, HttpRequest, HttpResponse, Responder};

use isomorphic_app::App;

const HTML_PLACEHOLDER: &str = "#HTML_INSERTED_HERE_BY_SERVER#";
const STATE_PLACEHOLDER: &str = "#INITIAL_STATE_JSON#";

fn index(req: &HttpRequest) -> impl Responder {
    let path = "/".to_string() + req.match_info().get("path").unwrap_or("");

    let app = App::new(
        req.query()
            .get("init")
            .map(|string| string.parse().expect("bad param"))
            .unwrap_or(1001),
        path,
    );
    let state = app.store.borrow();

    let html = format!("{}", include_str!("./index.html"));
    let html = html.replacen(HTML_PLACEHOLDER, &app.render().to_string(), 1);
    let html = html.replacen(STATE_PLACEHOLDER, &state.to_json(), 1);

    HttpResponse::Ok().content_type("text/html").body(html)
}

pub fn serve() {
    let build_dir = {
        // Development
        #[cfg(debug_assertions)]
        {
            format!("{}/../client/build", env!("CARGO_MANIFEST_DIR"))
        }

        #[cfg(not(debug_assertions))]
        {
            // Production
            format!("{}/../client/dist", env!("CARGO_MANIFEST_DIR"))
        }
    };

    let server = actix_web::server::new(move || {
        let app = actix_web::App::new();
        let app = app
            .resource("/", |r| r.f(index))
            // Serve wasm and js files and any other assets
            .handler("/static", fs::StaticFiles::new(&build_dir).unwrap())
            // All routes go back to our single index route since this is a single page app
            .resource("/{path}", |r| r.f(index));
        app
    });

    let server = server.bind("0.0.0.0:7878").unwrap();

    println!("Listening on port 7878");
    server.run();
}
