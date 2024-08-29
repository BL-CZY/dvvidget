use std::fs;

use crate::daemon::structs::DaemonEvt;
use once_cell::sync::Lazy;
use tokio::sync::broadcast;

pub static EXIT_BROADCAST: Lazy<broadcast::Sender<()>> = Lazy::new(|| broadcast::channel(2).0);

pub fn send_exit() -> Result<(), ()> {
    if let Err(_) = EXIT_BROADCAST.send(()) {
        return Err(());
    }

    Ok(())
}

pub async fn receive_exit() -> Result<(), ()> {
    if let Err(_) = EXIT_BROADCAST.subscribe().recv().await {
        return Err(());
    }

    Ok(())
}

pub fn shutdown() {
    if let Err(()) = send_exit() {
        println!("Failed to shutdown, force exiting");
        // remove the socket
        if let Err(e) = fs::remove_file(crate::daemon::server::DEFAULT_SOCKET_PATH) {
            println!(
                "No remaining socket file found at the default location {}: {}",
                crate::daemon::server::DEFAULT_SOCKET_PATH,
                e
            );
        }

        std::process::exit(1);
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
}

#[derive(Debug)]
pub enum ClientErr {
    CannotConnectServer,
    SerializeError(DaemonEvt, String),
    WriteErr(String),
}

pub fn vol_round(val: f64) -> f64 {
    val - val % 5.0
}
