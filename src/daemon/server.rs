use crate::utils::{shutdown, DaemonErr};
use std::fs;
use std::path::Path;
use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::unix::ReadHalf;
use tokio::net::UnixStream;
use tokio::sync::mpsc::UnboundedSender;

use super::structs::DaemonEvt;
use crate::utils::receive_exit;

pub const DEFAULT_SOCKET_PATH: &str = "/tmp/dvviget.sock";

pub async fn run_server(
    path: Option<String>,
    evt_sender: UnboundedSender<DaemonEvt>,
) -> Result<(), DaemonErr> {
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
        tokio::select! {
            Ok(()) = receive_exit() => {
                break;
            }

            Ok(res) = listener.accept() => {
                let stream: UnixStream = res.0;
                let new_sender = evt_sender.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, new_sender).await {
                        println!("Error reading the command: {:?}, ignoring", e)
                    }
                });
            }
        }
    }

    println!("shutting down the server..");
    drop(listener);
    fs::remove_file(socket_path).unwrap();

    Ok(())
}

// forwad the event to channel and return it
async fn handle_connection(
    mut stream: UnixStream,
    evt_sender: UnboundedSender<DaemonEvt>,
) -> Result<(), DaemonErr> {
    let (mut reader, mut writer) = stream.split();

    let evt: DaemonEvt = match read_cmd(&mut reader).await {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    if let Err(e) = evt_sender.send(evt.clone()) {
        return Err(DaemonErr::SendFailed(e.0));
    };

    if let Err(e) = writer.shutdown().await {
        return Err(DaemonErr::ShutdownFailed(e.to_string()));
    };

    if let DaemonEvt::ShutDown = evt {
        shutdown();
    }

    Ok(())
}

// the sender will send the struct in the following fashion:
/**
 * +----------------------------+-------------+
 * | u32 size in littlen endian | actual data |
 * +----------------------------+-------------+
 */
async fn read_cmd(reader: &mut ReadHalf<'_>) -> Result<DaemonEvt, DaemonErr> {
    let mut msg_len_buf = [0u8; 4];
    if let Err(e) = reader.read_exact(&mut msg_len_buf).await {
        return Err(DaemonErr::ReadingFailed(e.to_string()));
    };
    let msg_len: u32 = u32::from_le_bytes(msg_len_buf);

    let mut msg_buf = Vec::<u8>::with_capacity(msg_len as usize);

    // ensure we read all the info
    while msg_buf.len() < msg_len as usize {
        if let Err(e) = reader.read_buf(&mut msg_buf).await {
            return Err(DaemonErr::ReadingFailed(e.to_string()));
        };
    }

    Ok(match bincode::deserialize(&msg_buf) {
        Ok(val) => val,
        Err(e) => return Err(DaemonErr::DeserializeError(e.to_string())),
    })
}
