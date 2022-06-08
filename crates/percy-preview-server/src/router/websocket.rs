use crate::websocket::WebsocketConnections;
use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;
use axum::Extension;

pub(crate) const WEBSOCKET_ROUTE: &'static str = "/ws";

pub(crate) async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(websocket_connections): Extension<WebsocketConnections>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| async move {
        websocket_connections
            .connections
            .lock()
            .unwrap()
            .push(socket);
    })
}
