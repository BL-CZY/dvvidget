use std::path::Path;

use crate::{
    daemon::{
        server::default_socket_path,
        structs::{DaemonCmdClient, DaemonCmdType, DaemonRes},
    },
    utils::ClientErr,
};
use anyhow::Context;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

async fn send_to_stream(
    evt: DaemonCmdClient,
    mut stream: UnixStream,
) -> Result<UnixStream, ClientErr> {
    let evt_buf = match bincode::serialize(&evt).context("Failed to serialize command") {
        Ok(res) => res,
        Err(e) => return Err(ClientErr::SerializeError(evt, e.to_string())),
    };

    match stream.write(&(evt_buf.len() as u32).to_le_bytes()).await {
        Ok(_) => {}
        Err(e) => {
            return Err(ClientErr::WriteErr(e.to_string()));
        }
    }

    if let Err(e) = stream.write_all(&evt_buf).await {
        return Err(ClientErr::WriteErr(e.to_string()));
    }

    Ok(stream)
}

// the sender will send the struct in the following fashion:
/**
 * +----------------------------+-------------+
 * | u32 size in littlen endian | actual data |
 * +----------------------------+-------------+
 */
async fn read_res(mut stream: UnixStream) -> Result<DaemonRes, ClientErr> {
    let mut msg_len_buf = [0u8; 4];
    if let Err(e) = stream.read_exact(&mut msg_len_buf).await {
        return Err(ClientErr::ReadingFailed(e.to_string()));
    };
    let msg_len: u32 = u32::from_le_bytes(msg_len_buf);

    let mut msg_buf = Vec::<u8>::with_capacity(msg_len as usize);

    // ensure we read all the info
    while msg_buf.len() < msg_len as usize {
        if let Err(e) = stream
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

pub async fn send_evt_async(evt: DaemonCmdClient) -> Result<(), ClientErr> {
    let stream: UnixStream =
        if let Ok(res) = UnixStream::connect(Path::new(&default_socket_path())).await {
            res
        } else {
            return Err(ClientErr::CannotConnectServer);
        };

    let stream = send_to_stream(evt.clone(), stream).await?;

    if let DaemonCmdType::ShutDown = evt.cmd {
        println!("Signal sent");
        return Ok(());
    }

    let response = read_res(stream).await?;

    match response {
        DaemonRes::Failure(e) => println!("Failed: {}", e),
        DaemonRes::Success => println!("Success"),
        DaemonRes::GetVol(val) => println!("{}", val),
        DaemonRes::GetMute(val) => println!("{}", val),
        DaemonRes::GetBri(val) => println!("{}", val),
    }

    Ok(())
}
