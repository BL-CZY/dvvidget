use std::{fs, time::Duration};

use crate::daemon::structs::{DaemonCmd, DaemonEvt, DaemonRes};
use once_cell::sync::Lazy;
use tokio::sync::broadcast;

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

    if let Err(e) = send_exit() {
        println!("Failed to shutdown: {}, force exiting", e);
        // remove the socket
        if let Err(e) = fs::remove_file(crate::daemon::server::default_socket_path()) {
            println!(
                "No remaining socket file found at the default location {}: {}",
                crate::daemon::server::default_socket_path(),
                e
            );
        }

        std::process::exit(1);
    } else {
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
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
}

#[derive(Debug)]
pub enum ClientErr {
    CannotConnectServer,
    SerializeError(DaemonCmd, String),
    DeserializeError(String),
    ReadingFailed(String),
    WriteErr(String),
}

pub fn vol_round_down(val: f64) -> f64 {
    val - val % 5.0
}

pub fn vol_round_up(val: f64) -> f64 {
    if val % 5.0 == 0.0 {
        val
    } else {
        val + (5.0 - val % 5.0)
    }
}
