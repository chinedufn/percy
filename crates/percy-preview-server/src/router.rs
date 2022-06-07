use crate::router::websocket::WEBSOCKET_ROUTE;
use crate::{make_html, ServerConfig};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, get_service};
use axum::{Extension, Router};
use std::sync::Arc;
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

mod websocket;

pub(crate) fn create_router(config: ServerConfig) -> Router {
    let html = make_html(
        "Percy Preview App",
        &config.wasm_file_name,
        &config.javascript_file_name,
    );

    Router::new()
        .fallback(get(catch_all))
        .nest(
            "/static",
            get_service(ServeDir::new(config.assets_dir)).handle_error(handle_error),
        )
        .route(WEBSOCKET_ROUTE, get(websocket::websocket_handler))
        .layer(Extension(Arc::new(CatchAllHtml(html))))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
}

struct CatchAllHtml(String);

async fn catch_all(Extension(html): Extension<Arc<CatchAllHtml>>) -> Html<String> {
    Html(html.0.clone())
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
