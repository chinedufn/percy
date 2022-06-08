//! The server that powers the Percy Preview app.

#![deny(missing_docs)]

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::mpsc::channel;

use crate::filesystem_watcher::{
    start_fs_notification_receiver_thread, start_fs_notification_sender_thread,
    NotificationReceiverConfig, NotificationSenderConfig,
};
use crate::websocket::WebsocketConnections;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use self::config::ServerConfig;

mod config;
mod router;

mod filesystem_watcher;
mod websocket;

/// Start the Percy Preview server.
pub async fn start_server(config: ServerConfig) {
    init_tracing();

    let crate_library_name = get_crate_library_name(&config.crate_dir);

    let wasm_file = format!("{}_bg.wasm", crate_library_name);
    let javascript_file = format!("{}.js", crate_library_name);

    let html = make_html("Percy Preview App", &wasm_file, &javascript_file);
    std::fs::write(config.out_dir.join("index.html"), &html).unwrap();

    let websocket_connections = WebsocketConnections::default();

    let (refresh_tx, refresh_rx) = channel();
    start_fs_notification_sender_thread(NotificationSenderConfig {
        crate_dir: config.crate_dir.clone(),
        target_dir: config.target_dir.clone(),
        out_dir: config.out_dir.clone(),
        refresh_tx,
    });
    start_fs_notification_receiver_thread(NotificationReceiverConfig {
        connections: websocket_connections.clone(),
        refresh_rx,
    });

    let port = config.port;
    let app = router::create_router(config, html, websocket_connections);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Percy Preview Server listening on port {}", port);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "percy_preview_server=info,tower_http=info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// /path/to/some-crate -> some_crate
fn get_crate_library_name(crate_dir: &PathBuf) -> String {
    crate_dir
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .replace("-", "_")
}

fn make_html(title: &str, wasm: &str, javascript: &str) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html lang="en" style="height: 100%; width: 100%; margin: 0; padding: 0;">
  <head>
      <meta charset="UTF-8">
      <meta name="viewport" content="width=device-width, initial-scale=1">
      <link rel="stylesheet" type="text/css" href="app.css"/>
      <title>{title}</title>
  </head>
  <body style='margin: 0; padding: 0; width: 100%; height: 100%;'>
      <div id='app-mount' style='margin: 0; padding: 0; width: 100%; height: 100%;'></div>
  
      <script type="module">
          import init, {{start_component_preview}} from '/static/{javascript}'
      
          async function run () {{
              await init('/static/{wasm}')
              start_component_preview('#app-mount')
          }}
      
          run()
      </script>
      <script>
        const loc = window.location
        const wsUrl = "ws://" + loc.host + "/ws";
        
        const socket = new WebSocket(wsUrl);
        
        socket.addEventListener('message', function (event) {{
            console.log("Reloading")
            window.location.reload();
        }});
      
      </script>
  </body>
</html>
"#
    )
}
