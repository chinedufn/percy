use axum::response::IntoResponse;

pub(crate) const WEBSOCKET_ROUTE: &'static str = "/ws";

pub(crate) async fn websocket_handler() -> impl IntoResponse {
    unimplemented!()
}
