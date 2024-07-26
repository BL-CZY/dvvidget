pub mod info;
pub mod renderer;
pub mod server;

use crate::utils::DaemonErr;
use std::path::Path;
use tokio;

const DEFAULT_SOCKET_PATH: &str = "/tmp/dvviget.sock";

pub fn start_daemon(path: Option<String>) -> Result<(), DaemonErr> {
    let socket_path: String = if let Some(p) = path {
        p
    } else {
        DEFAULT_SOCKET_PATH.into()
    };

    if Path::new(&socket_path).exists() {
        return Err(DaemonErr::ServerAlreadyRunning);
    }

    let listener = if let Ok(res) = tokio::net::UnixListener::bind(Path::new(&socket_path)) {
        res
    } else {
        return Err(DaemonErr::InitServerFailed);
    };

    Ok(())
}
