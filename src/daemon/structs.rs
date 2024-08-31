use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum DaemonCmd {
    CloseWindow,
    ShutDown,
    GetVol,
    SetVol(u32),
    IncVol(u32),
    DecVol(u32),
}

#[derive(Debug, Clone)]
pub struct DaemonEvt {
    pub evt: DaemonCmd,
    pub sender: UnboundedSender<DaemonRes>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DaemonRes {
    VolGet(f64),
    Success,
    Failure(String),
}
