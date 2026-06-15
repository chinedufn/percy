use isomorphic_app::App;
use worker::*;

const DEFAULT_INIT: u32 = 1001;
const HTML_PLACEHOLDER: &str = "#HTML_INSERTED_HERE_BY_SERVER#";
const STATE_PLACEHOLDER: &str = "#INITIAL_STATE_JSON#";
const INDEX_HTML: &str = include_str!("../../server/src/index.html");

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let path = req.path();
    if path.starts_with("/static/") {
        return env.assets("ASSETS")?.fetch_request(req).await;
    }

    match path.as_str() {
        "/" | "/contributors" => {
            let init = match initial_count_from_query(req.url()?.query()) {
                Ok(init) => init,
                Err(()) => return Response::error("Invalid init query parameter", 400),
            };

            Response::from_html(render_html(path, init))
        }
        _ => Response::error("Not found", 404),
    }
}

fn render_html(path: String, init: Option<u32>) -> String {
    let app = App::new(init.unwrap_or(DEFAULT_INIT), path);
    let state = app.store.borrow();

    let html = INDEX_HTML.replacen(HTML_PLACEHOLDER, &app.render().to_string(), 1);
    html.replacen(STATE_PLACEHOLDER, &state.to_json(), 1)
}

fn initial_count_from_query(query: Option<&str>) -> std::result::Result<Option<u32>, ()> {
    let Some(query) = query else {
        return Ok(None);
    };

    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        if key == "init" {
            if value.is_empty() {
                return Err(());
            }

            return value.parse::<u32>().map(Some).map_err(|_| ());
        }
    }

    Ok(None)
}
