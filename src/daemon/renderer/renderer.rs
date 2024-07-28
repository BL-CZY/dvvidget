use tokio::sync::mpsc::UnboundedReceiver;

use crate::{daemon::structs::DaemonEvt, utils::DaemonErr};

pub fn init_gtk_async(mut evt_receiver: UnboundedReceiver<DaemonEvt>) -> Result<(), DaemonErr> {
    glib::MainContext::default().spawn_local(async move {
        while let Some(evt) = evt_receiver.recv().await {
            // handle the event
        }
    });
    Ok(())
}
