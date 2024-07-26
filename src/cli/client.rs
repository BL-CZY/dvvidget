use std::path::Path;

use crate::utils::ClientErr;
use tokio::net::UnixStream;

const DEFAULT_SOCKET_PATH: &str = "/tmp/dvviget.sock";

pub async fn connect_server() -> Result<(), ClientErr> {
    let stream = if let Ok(res) = UnixStream::connect(Path::new(DEFAULT_SOCKET_PATH)).await {
        res
    } else {
        return Err(ClientErr::CannotConnectServer);
    };

    Ok(())
}
