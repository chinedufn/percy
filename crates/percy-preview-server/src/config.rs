use std::path::PathBuf;

/// Configure the Percy Preview server.
pub struct ServerConfig {
    /// The port to listen on.
    pub port: u16,
    /// The crate to preview.
    pub crate_dir: PathBuf,
    /// The target directory to use as the CARGO_TARGET_DIR when building the previewed crate.
    pub target_dir: PathBuf,
    /// The directory where the wasm file and JS file live.
    pub out_dir: PathBuf,
}
