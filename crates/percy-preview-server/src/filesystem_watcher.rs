use crate::WebsocketConnections;
use axum::extract::ws::Message;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, RecvError, SendError, Sender};
use std::time::Duration;

// 0.2 is somewhat arbitrarily chosen.
const WATCHER_DELAY_SECS: f32 = 0.2;

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
        let mut watcher = watcher(tx, Duration::from_secs_f32(WATCHER_DELAY_SECS)).unwrap();

        watcher
            .watch(&config.crate_dir, RecursiveMode::Recursive)
            .unwrap();

        build(&config.crate_dir, &config.target_dir, &config.out_dir);

        loop {
            let event = rx.recv().unwrap();

            match event {
                DebouncedEvent::NoticeWrite(_)
                | DebouncedEvent::NoticeRemove(_)
                | DebouncedEvent::Create(_)
                | DebouncedEvent::Write(_)
                | DebouncedEvent::Remove(_)
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
                DebouncedEvent::Rescan | DebouncedEvent::Chmod(_) | DebouncedEvent::Error(_, _) => {
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
            while let recv = config.refresh_rx.recv() {
                match config.refresh_rx.recv() {
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

    cargo_build(crate_dir, target_dir)?;
    wasm_bindgen_build(&project_library_name, target_dir, out_dir)
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
