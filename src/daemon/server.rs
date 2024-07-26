use crate::utils::DaemonErr;
use std::fs;
use std::path::Path;
use tokio;
use tokio::net::UnixStream;

const DEFAULT_SOCKET_PATH: &str = "/tmp/dvviget.sock";

pub async fn start_server(path: Option<String>) -> Result<(), DaemonErr> {
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

    loop {
        let stream: UnixStream = if let Ok(res) = listener.accept().await {
            res.0
        } else {
            break;
        };
    }

    drop(listener);
    fs::remove_file(socket_path).unwrap();

    Ok(())
}
