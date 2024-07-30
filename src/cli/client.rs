use std::path::Path;

use crate::{daemon::structs::DaemonEvt, utils::ClientErr};
use anyhow::Context;
use tokio::{io::AsyncWriteExt, net::UnixStream};

const DEFAULT_SOCKET_PATH: &str = "/tmp/dvviget.sock";

async fn send_to_stream(evt: DaemonEvt, mut stream: UnixStream) -> Result<(), ClientErr> {
    let evt_buf = match bincode::serialize(&evt).context("Failed to serialize command") {
        Ok(res) => res,
        Err(e) => return Err(ClientErr::SerializeError(evt, e.to_string())),
    };

    match stream.write(&(evt_buf.len() as u32).to_le_bytes()).await {
        Ok(res) => {
            if res != evt_buf.len() as usize {
                println!("Size don't match");
            }
        }
        Err(e) => {
            return Err(ClientErr::WriteErr(e.to_string()));
        }
    }

    if let Err(e) = stream.write_all(&evt_buf).await {
        return Err(ClientErr::WriteErr(e.to_string()));
    }

    Ok(())
}

pub async fn send_evt_async(evt: DaemonEvt) -> Result<(), ClientErr> {
    let stream = if let Ok(res) = UnixStream::connect(Path::new(DEFAULT_SOCKET_PATH)).await {
        res
    } else {
        return Err(ClientErr::CannotConnectServer);
    };

    if let Err(e) = send_to_stream(evt, stream).await {
        return Err(e);
    }

    Ok(())
}
