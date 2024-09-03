use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum DaemonCmd {
    ShutDown,
    RegVolClose(f64),
    ExecVolClose(f64),
    Vol(Vol),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Vol {
    Get,
    Set(u32),
    Inc(u32),
    Dec(u32),
    Close,
    Open,
    OpenTime(f64),
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
