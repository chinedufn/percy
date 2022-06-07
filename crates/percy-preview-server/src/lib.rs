//! The server that powers the Percy Preview app.

#![deny(missing_docs)]

use std::net::SocketAddr;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use self::config::ServerConfig;

mod config;
mod router;

/// Start the Percy Preview server.
pub async fn start_server(config: ServerConfig) {
    init_tracing();

    let html = make_html(
        "Percy Preview App",
        &config.wasm_file_name,
        &config.javascript_file_name,
    );
    std::fs::write(config.assets_dir.join("index.html"), html).unwrap();

    let port = config.port;
    let app = router::create_router(config);

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
  </body>
</html>
"#
    )
}
