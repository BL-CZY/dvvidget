use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum DaemonCmd {
    ShutDown,
    Vol(Vol),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Vol {
    Get,
    Mute,
    SetRough(f64),
    Set(f64),
    Dec(f64),
    Inc(f64),
    Close,
    Open,
    OpenTimed(f64),
}

#[derive(Debug, Clone)]
pub struct DaemonEvt {
    pub evt: DaemonCmd,
    pub sender: Option<UnboundedSender<DaemonRes>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DaemonRes {
    VolGet(f64),
    Success,
    Failure(String),
}
