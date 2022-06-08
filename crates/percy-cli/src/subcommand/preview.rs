use clap::Parser;
use percy_preview_server::{start_server, ServerConfig};
use std::path::PathBuf;
use tokio::runtime::Runtime;

const DEFAULT_PORT: u16 = 16500;

#[derive(Debug, Parser)]
pub(crate) struct Preview {
    /// The crate to preview
    crate_dir: PathBuf,
    /// The port to serve the preview on.
    #[clap(short, long)]
    port: Option<u16>,
    /// The value to set for the CARGO_TARGET_DIR environment variable when building the crate.
    #[clap(short, long)]
    target: PathBuf,
}

impl Preview {
    pub fn run(self) -> anyhow::Result<()> {
        let port = self.port.unwrap_or(DEFAULT_PORT);

        let out_dir = self.target.join(format!(
            "percy-preview-{}",
            self.crate_dir.file_name().unwrap().to_str().unwrap()
        ));
        std::fs::create_dir_all(&out_dir).unwrap();

        let server_config = ServerConfig {
            port,
            crate_dir: self.crate_dir,
            target_dir: self.target,
            out_dir,
        };

        let runtime = Runtime::new().unwrap();
        runtime.block_on(async move { start_server(server_config).await });

        Ok(())
    }
}
