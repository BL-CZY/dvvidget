pub mod info;
pub mod renderer;
pub mod server;
pub mod structs;
use crate::utils::DaemonErr;
use structs::DaemonCmd;

use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub async fn start_daemon(path: Option<String>) -> Result<(), DaemonErr> {
    let (evt_sender, evt_receiver): (UnboundedSender<DaemonCmd>, UnboundedReceiver<DaemonCmd>) =
        mpsc::unbounded_channel();

    if let Err(e) = server::run_server(path, evt_sender.clone()).await {
        return Err(e);
    }
    Ok(())
}
