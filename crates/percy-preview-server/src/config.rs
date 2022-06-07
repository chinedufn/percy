use std::path::PathBuf;

/// Configure the Percy Preview server.
pub struct ServerConfig {
    /// The port to listen on.
    pub port: u16,
    /// THe directory where the wasm file and JS file live.
    pub assets_dir: PathBuf,
    /// The name of the application's wasm file.
    pub wasm_file_name: String,
    /// The name of the application's wasm-bindgen auto-generated JavaScript file.
    pub javascript_file_name: String,
}
