use crate::WebsocketConnections;
use axum::extract::ws::Message;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use sunbeam_ir::{Classes, SunbeamConfig};

// 0.3 is somewhat arbitrarily chosen.
const WATCHER_DEBOUNCE_SECS: f32 = 0.3;

pub(crate) struct NotificationSenderConfig {
    pub crate_dir: PathBuf,
    pub target_dir: PathBuf,
    pub out_dir: PathBuf,
    pub refresh_tx: Sender<()>,
}

pub(crate) struct NotificationReceiverConfig {
    pub connections: WebsocketConnections,
    pub refresh_rx: Receiver<()>,
}

pub(crate) fn start_fs_notification_sender_thread(
    config: NotificationSenderConfig,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs_f32(WATCHER_DEBOUNCE_SECS)).unwrap();

        watcher
            .watch(&config.crate_dir, RecursiveMode::Recursive)
            .unwrap();

        build(&config.crate_dir, &config.target_dir, &config.out_dir);

        loop {
            let event = rx.recv().unwrap();

            match event {
                DebouncedEvent::Create(_)
                | DebouncedEvent::Write(_)
                | DebouncedEvent::Rename(_, _) => {
                    match build(&config.crate_dir, &config.target_dir, &config.out_dir) {
                        Ok(_) => {
                            match config.refresh_tx.send(()) {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("{}", e);
                                }
                            };
                        }
                        Err(err) => {
                            eprintln!("{}", err)
                        }
                    }
                }
                _ => {
                    // Nothing to do...
                }
            }
        }
    })
}

pub(crate) fn start_fs_notification_receiver_thread(config: NotificationReceiverConfig) {
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async move {
            loop {
                let recv = config.refresh_rx.recv();
                match recv {
                    Ok(_) => {
                        let mut connections = config.connections.connections.lock().unwrap();

                        let mut idx = 0;
                        while idx < connections.len() {
                            match connections[idx].send(Message::Binary(vec![1, 2, 3])).await {
                                Ok(_) => {
                                    idx += 1;
                                }
                                Err(_) => {
                                    connections.remove(idx);
                                }
                            };
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        return;
                    }
                };
            }
        });
    });
}

fn build(crate_dir: &PathBuf, target_dir: &PathBuf, out_dir: &PathBuf) -> anyhow::Result<()> {
    let project_library_name = crate_dir.file_name().unwrap().to_str().unwrap();
    let project_library_name = project_library_name.replace("-", "_");

    cargo_build(crate_dir, target_dir, out_dir)?;
    wasm_bindgen_build(&project_library_name, target_dir, out_dir)?;
    run_sunbeam_build(&crate_dir, &out_dir)
}

fn cargo_build(
    project_dir: &PathBuf,
    target_dir: &PathBuf,
    out_dir: &PathBuf,
) -> anyhow::Result<()> {
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
        .env("SUNBEAM_DIR", out_dir.canonicalize().unwrap())
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

fn run_sunbeam_build(crate_dir: &PathBuf, out_dir: &PathBuf) -> anyhow::Result<()> {
    let crate_sunbeam_config =
        serde_yaml::from_str(&std::fs::read_to_string(crate_dir.join("Sunbeam.yml")).unwrap())?;

    let all_sunbeam = out_dir.join("all_sunbeam_css.rs");

    if !all_sunbeam.exists() {
        return Err(anyhow::anyhow!(
            "All sunbeam file does not exist: {:?}",
            all_sunbeam
        ));
    }

    let mut classes =
        sunbeam_build::parse_rust_files(std::iter::once(all_sunbeam), &crate_sunbeam_config)?;

    let percy_preview_app_sunbeam_config: SunbeamConfig =
        serde_yaml::from_str(percy_preview_app::all_sunbeam_css::SUNBEAM_CONFIG_YML).unwrap();
    for class in percy_preview_app::all_sunbeam_css::all() {
        classes.extend(Classes::parse_str(class, &percy_preview_app_sunbeam_config).unwrap());
    }

    std::fs::write(out_dir.join("app.css"), classes.to_css_file_contents())?;

    Ok(())
}
