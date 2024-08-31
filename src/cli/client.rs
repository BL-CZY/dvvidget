use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use crate::{
    daemon::structs::{DaemonCmd, DaemonRes},
    utils::ClientErr,
};
use anyhow::Context;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

const DEFAULT_SOCKET_PATH: &str = "/tmp/dvviget.sock";

async fn send_to_stream(evt: DaemonCmd, stream: Arc<Mutex<UnixStream>>) -> Result<(), ClientErr> {
    let evt_buf = match bincode::serialize(&evt).context("Failed to serialize command") {
        Ok(res) => res,
        Err(e) => return Err(ClientErr::SerializeError(evt, e.to_string())),
    };

    match stream
        .lock()
        .unwrap()
        .write(&(evt_buf.len() as u32).to_le_bytes())
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(ClientErr::WriteErr(e.to_string()));
        }
    }

    if let Err(e) = stream.lock().unwrap().write_all(&evt_buf).await {
        return Err(ClientErr::WriteErr(e.to_string()));
    }

    Ok(())
}

// the sender will send the struct in the following fashion:
/**
 * +----------------------------+-------------+
 * | u32 size in littlen endian | actual data |
 * +----------------------------+-------------+
 */
async fn read_res(stream: Arc<Mutex<UnixStream>>) -> Result<DaemonRes, ClientErr> {
    let mut msg_len_buf = [0u8; 4];
    if let Err(e) = stream.lock().unwrap().read_exact(&mut msg_len_buf).await {
        return Err(ClientErr::ReadingFailed(e.to_string()));
    };
    let msg_len: u32 = u32::from_le_bytes(msg_len_buf);

    let mut msg_buf = Vec::<u8>::with_capacity(msg_len as usize);

    // ensure we read all the info
    while msg_buf.len() < msg_len as usize {
        if let Err(e) = stream
            .lock()
            .unwrap()
            .read_buf(&mut msg_buf)
            .await
            .context("Failed to read the command from the client")
        {
            return Err(ClientErr::ReadingFailed(e.to_string()));
        };
    }

    Ok(
        match bincode::deserialize(&msg_buf).context("Failed to deserialize command") {
            Ok(val) => val,
            Err(e) => return Err(ClientErr::DeserializeError(e.to_string())),
        },
    )
}

pub async fn send_evt_async(evt: DaemonCmd) -> Result<(), ClientErr> {
    let stream: Arc<Mutex<UnixStream>> =
        if let Ok(res) = UnixStream::connect(Path::new(DEFAULT_SOCKET_PATH)).await {
            Arc::new(Mutex::new(res))
        } else {
            return Err(ClientErr::CannotConnectServer);
        };

    send_to_stream(evt, stream.clone()).await?;

    let response = read_res(stream.clone()).await?;

    match response {
        DaemonRes::Failure(e) => println!("Failed: {}", e),
        DaemonRes::Success => println!("Success"),
        DaemonRes::VolGet(val) => println!("{}", val),
    }

    Ok(())
}
