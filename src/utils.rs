use std::{path::PathBuf, sync::atomic::AtomicBool, time::Duration};

use crate::daemon::structs::{DaemonCmdClient, DaemonEvt, DaemonRes};
use gtk4::Image;
use once_cell::sync::Lazy;
use tokio::sync::broadcast;

pub fn cache_dir() -> PathBuf {
    let mut result = PathBuf::from(std::env::var("HOME").expect("Cannot find home dir"));
    result.push(".cache/dvvidget");

    result
}

/// returns a list of paths from XDG_DATA_DIRS and attach $HOME/.local/share/applications/ at the
/// end
pub fn get_paths() -> Vec<PathBuf> {
    let mut result = std::env::var("XDG_DATA_DIRS")
        .unwrap()
        .split(":")
        .filter_map(|s| {
            let mut res = PathBuf::from(s);
            res.push("applications/");

            if res.is_dir() {
                Some(res)
            } else {
                None
            }
        })
        .collect::<Vec<PathBuf>>();

    if let Ok(v) = std::env::var("HOME") {
        let mut p = PathBuf::from(&v);
        p.push(".local/share/applications/");
        result.push(p);
    }

    result
}

#[derive(Clone, Copy)]
pub enum DisplayBackend {
    Wayland,
    X11,
}

pub fn detect_display() -> DisplayBackend {
    use std::env;
    let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_default();

    if session_type.contains("x11") {
        DisplayBackend::X11
    } else if session_type.contains("wayland")
        && !env::var("WAYLAND_DISPLAY").unwrap_or_default().is_empty()
    {
        DisplayBackend::Wayland
    } else {
        println!("No display session detected, exiting...");
        std::process::exit(1);
    }
}

#[derive(Clone)]
pub enum ExitType {
    Exit,
    Restart,
}

pub static EXIT_BROADCAST: Lazy<broadcast::Sender<ExitType>> =
    Lazy::new(|| broadcast::channel(2).0);

pub static EXIT_SENT: AtomicBool = AtomicBool::new(false);

pub fn send_exit() -> Result<(), String> {
    if EXIT_SENT.load(std::sync::atomic::Ordering::SeqCst) {
        return Err("Sent already".into());
    }

    if let Err(e) = EXIT_BROADCAST.send(ExitType::Exit) {
        return Err(e.to_string());
    }

    EXIT_SENT.store(true, std::sync::atomic::Ordering::SeqCst);

    Ok(())
}

pub async fn receive_exit() -> Result<ExitType, ()> {
    match EXIT_BROADCAST.subscribe().recv().await {
        Ok(t) => Ok(t),
        Err(_) => Err(()),
    }
}

pub fn shutdown(msg: &str) -> ! {
    println!("{}", msg);
    send_exit().unwrap_or_else(|e| {
        println!("Failed to shutdown: {}, force exiting", e);
        std::process::exit(1);
    });

    std::thread::sleep(Duration::from_secs(3));
    println!("Timeout, force exiting");
    std::process::exit(0);
}

#[derive(Debug)]
pub enum DaemonErr {
    InitServerFailed,
    ServerAlreadyRunning,
    ReadingFailed(String),
    DeserializeError(String),
    SendFailed(DaemonEvt),
    ShutdownFailed(String),
    WriteErr(String),
    SerializeError(DaemonRes, String),
    FileWatchError(String),
    CannotFindWidget,
}

#[derive(Debug)]
pub enum ClientErr {
    CannotConnectServer,
    SerializeError(DaemonCmdClient, String),
    DeserializeError(String),
    ReadingFailed(String),
    WriteErr(String),
}

pub fn round_down(val: f64) -> f64 {
    val - val % 5.0
}

pub fn set_svg(pic: &Image, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    pic.set_from_file(Some(path));
    Ok(())
}
