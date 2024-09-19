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
    SetMute(bool),
    ToggleMute,
    GetMute,
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
    GetVol(f64),
    GetMute(bool),
    Success,
    Failure(String),
}
