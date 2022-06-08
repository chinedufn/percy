use axum::extract::ws::WebSocket;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub(crate) struct WebsocketConnections {
    pub(crate) connections: Arc<Mutex<Vec<WebSocket>>>,
}
