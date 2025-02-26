use std::{path::PathBuf, time::Duration};

use crate::daemon::structs::{DaemonCmd, DaemonEvt, DaemonRes};
use gtk4::Image;
use once_cell::sync::Lazy;
use tokio::sync::broadcast;

pub fn get_paths() -> Vec<PathBuf> {
    std::env::var("XDG_DATA_DIRS")
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
        .collect::<Vec<PathBuf>>()
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

pub static EXIT_BROADCAST: Lazy<broadcast::Sender<()>> = Lazy::new(|| broadcast::channel(2).0);

pub fn send_exit() -> Result<(), String> {
    if let Err(e) = EXIT_BROADCAST.send(()) {
        return Err(e.to_string());
    }

    Ok(())
}

pub async fn receive_exit() -> Result<(), ()> {
    if (EXIT_BROADCAST.subscribe().recv().await).is_err() {
        return Err(());
    }

    Ok(())
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
    SerializeError(DaemonCmd, String),
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
