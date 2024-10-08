use crate::utils::{shutdown, DaemonErr};
use anyhow::Context;
use std::fs;
use std::path::Path;
use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::unix::ReadHalf;
use tokio::net::UnixStream;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use super::structs::{DaemonCmd, DaemonEvt, DaemonRes};
use crate::utils::receive_exit;

pub fn default_socket_path() -> String {
    let val = env!("CARGO_PKG_VERSION").replace(".", "-");
    format!("/tmp/dvvidget-{}.sock", val)
}

async fn is_active_socket(path: &str) -> bool {
    UnixStream::connect(Path::new(path)).await.is_ok()
}

pub async fn run_server(
    path: Option<String>,
    evt_sender: UnboundedSender<DaemonEvt>,
) -> Result<(), DaemonErr> {
    let socket_path: String = if let Some(p) = path {
        p
    } else {
        default_socket_path()
    };

    if Path::new(&socket_path).exists() {
        if is_active_socket(&socket_path).await {
            shutdown("There is an existing server running");
        } else {
            println!("Found an inactive socket, cleaning...");
            fs::remove_file(&socket_path).unwrap();
        }
    }

    let listener = if let Ok(res) = tokio::net::UnixListener::bind(Path::new(&socket_path)) {
        res
    } else {
        shutdown("Failed to initialize the server");
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
    let (res_sender, mut res_receiver): (UnboundedSender<DaemonRes>, UnboundedReceiver<DaemonRes>) =
        mpsc::unbounded_channel();

    let evt: DaemonCmd = match read_cmd(&mut reader).await {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let cmd = DaemonEvt {
        evt: evt.clone(),
        sender: Some(res_sender),
    };

    println!("Event receiverd from client: {:?}", evt);

    if let DaemonCmd::ShutDown = evt {
        shutdown("Shutting down...");
    }

    if let Err(e) = evt_sender.send(cmd.clone()) {
        return Err(DaemonErr::SendFailed(e.0));
    };

    if let Some(res) = res_receiver.recv().await {
        let evt_buf = match bincode::serialize(&res).context("Failed to serialize command") {
            Ok(res) => res,
            Err(e) => return Err(DaemonErr::SerializeError(res, e.to_string())),
        };

        match writer.write(&(evt_buf.len() as u32).to_le_bytes()).await {
            Ok(_) => {}
            Err(e) => {
                return Err(DaemonErr::WriteErr(e.to_string()));
            }
        }

        if let Err(e) = writer.write_all(&evt_buf).await {
            return Err(DaemonErr::WriteErr(e.to_string()));
        }
    } else {
        println!("Cant ");
    }

    if let Err(e) = writer.shutdown().await {
        return Err(DaemonErr::ShutdownFailed(e.to_string()));
    };

    Ok(())
}

// the sender will send the struct in the following fashion:
/**
 * +----------------------------+-------------+
 * | u32 size in littlen endian | actual data |
 * +----------------------------+-------------+
 */
async fn read_cmd(reader: &mut ReadHalf<'_>) -> Result<DaemonCmd, DaemonErr> {
    let mut msg_len_buf = [0u8; 4];
    if let Err(e) = reader.read_exact(&mut msg_len_buf).await {
        return Err(DaemonErr::ReadingFailed(e.to_string()));
    };
    let msg_len: u32 = u32::from_le_bytes(msg_len_buf);

    let mut msg_buf = Vec::<u8>::with_capacity(msg_len as usize);

    // ensure we read all the info
    while msg_buf.len() < msg_len as usize {
        if let Err(e) = reader
            .read_buf(&mut msg_buf)
            .await
            .context("Failed to read the command from the client")
        {
            return Err(DaemonErr::ReadingFailed(e.to_string()));
        };
    }

    Ok(
        match bincode::deserialize(&msg_buf).context("Failed to deserialize command") {
            Ok(val) => val,
            Err(e) => return Err(DaemonErr::DeserializeError(e.to_string())),
        },
    )
}
