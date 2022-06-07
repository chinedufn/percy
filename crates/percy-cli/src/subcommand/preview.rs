use clap::Parser;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use percy_preview_server::{start_server, ServerConfig};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;
use tempfile::TempDir;
use tokio::runtime::Runtime;

const DEFAULT_PORT: u16 = 16500;
// 0.2 is somewhat arbitrarily chosen.
const WATCHER_DELAY_SECS: f32 = 0.2;

#[derive(Debug, Parser)]
pub(crate) struct Preview {
    /// The crate to preview
    project: PathBuf,
    /// The port to serve the preview on.
    #[clap(short, long)]
    port: Option<u16>,
    /// The value to set for the CARGO_TARGET_DIR environment variable when building the crate.
    #[clap(short, long)]
    target: PathBuf,
}

impl Preview {
    pub fn run(self) -> anyhow::Result<()> {
        let outdir = TempDir::new().unwrap().into_path();

        let port = self.port.unwrap_or(DEFAULT_PORT);

        let (tx, rx) = channel();

        let mut watcher = watcher(tx, Duration::from_secs_f32(WATCHER_DELAY_SECS)).unwrap();

        watcher
            .watch(&self.project, RecursiveMode::Recursive)
            .unwrap();

        let server_config = ServerConfig {
            port,
            assets_dir: outdir.clone(),
            wasm_file_name: format!("{}_bg.wasm", &self.project_library_name()),
            javascript_file_name: format!("{}.js", &self.project_library_name()),
        };

        let _preview_server_thread = std::thread::spawn(move || {
            let runtime = Runtime::new().unwrap();
            runtime.block_on(async move { start_server(server_config).await });
        });

        self.build(&outdir)?;

        loop {
            let event = rx.recv()?;

            match event {
                DebouncedEvent::NoticeWrite(_)
                | DebouncedEvent::NoticeRemove(_)
                | DebouncedEvent::Create(_)
                | DebouncedEvent::Write(_)
                | DebouncedEvent::Remove(_)
                | DebouncedEvent::Rename(_, _) => {
                    self.build(&outdir)?;
                }
                DebouncedEvent::Rescan | DebouncedEvent::Chmod(_) => {
                    // Nothing to do...
                }
                DebouncedEvent::Error(err, _) => {
                    return Err(err)?;
                }
            }
        }
    }

    fn build(&self, outdir: &PathBuf) -> anyhow::Result<()> {
        println!("Rebuilding project.");
        build(
            &self.project,
            &self.project_library_name(),
            &self.target,
            &outdir,
        )
    }

    // some-crate -> some_crate
    fn project_library_name(&self) -> String {
        let project_name = self.project.file_name().unwrap().to_str().unwrap();
        project_name.replace("-", "_")
    }
}

fn build(
    project_dir: &PathBuf,
    project_library_name: &str,
    target_dir: &PathBuf,
    out_dir: &PathBuf,
) -> anyhow::Result<()> {
    cargo_build(project_dir, target_dir)?;
    wasm_bindgen_build(project_library_name, target_dir, out_dir)
}

fn cargo_build(project_dir: &PathBuf, target_dir: &PathBuf) -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");

    let output = cmd
        .arg("build")
        .args(&[
            "--manifest-path",
            project_dir.join("Cargo.toml").to_str().unwrap(),
        ])
        .args(&["--target", "wasm32-unknown-unknown"])
        .args(&["--features", "preview"])
        .env("CARGO_TARGET_DIR", target_dir)
        .spawn()?
        .wait_with_output()?;

    if output.status.success() {
        Ok(())
    } else {
        return Err(anyhow::anyhow!(
            "{}",
            String::from_utf8(output.stderr).unwrap()
        ));
    }
}

fn wasm_bindgen_build(
    project_library_name: &str,
    target: &PathBuf,
    out_dir: &PathBuf,
) -> anyhow::Result<()> {
    let wasm_path = target.join(format!(
        "wasm32-unknown-unknown/debug/{}.wasm",
        project_library_name
    ));

    let mut cmd = Command::new("wasm-bindgen");

    let output = cmd
        .arg(wasm_path)
        .arg("--no-typescript")
        .args(&["--target", "web"])
        .args(&["--out-dir", out_dir.to_str().unwrap()])
        .arg("--debug")
        .spawn()?
        .wait_with_output()?;

    if output.status.success() {
        Ok(())
    } else {
        return Err(anyhow::anyhow!(
            "{}",
            String::from_utf8(output.stderr).unwrap()
        ));
    }
}
