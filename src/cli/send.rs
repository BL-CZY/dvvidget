use crate::{daemon::structs::DaemonCmdClient, utils::ClientErr};

use super::client;

pub fn send_evt(evt: DaemonCmdClient) -> Result<(), ClientErr> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .thread_name("dvvidget client")
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        if let Err(e) = client::send_evt_async(evt).await {
            println!("Error: {:?}", e);
        }
    });

    Ok(())
}
