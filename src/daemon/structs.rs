use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use super::renderer::dvoty::DvotyEntry;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DaemonCmdType {
    ShutDown,
    Vol(Vol),
    Bri(Bri),
    Dvoty(Dvoty),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MonitorClient {
    All,
    One(usize),
}

impl MonitorClient {
    pub fn get_val(&self, monitor_count: usize) -> Vec<usize> {
        match self {
            Self::All => (0..monitor_count).collect(),
            Self::One(num) => vec![*num],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DaemonCmdClient {
    pub monitor: MonitorClient,
    pub cmd: DaemonCmdType,
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Bri {
    Get,
    SetRough(f64),
    Set(f64),
    Dec(f64),
    Inc(f64),
    Close,
    Open,
    OpenTimed(f64),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Dvoty {
    AddEntry(DvotyEntry),
    Update(String, Vec<PathBuf>),
    SetScroll(f64),
    ScrollEnd,
    ScrollStart,
    IncEntryIndex,
    DecEntryIndex,
    TriggerEntry,
    Close,
    Open,
    Toggle,
}

#[derive(Debug, Clone)]
pub struct DaemonEvt {
    pub evt: DaemonCmdType,
    pub sender: Option<UnboundedSender<DaemonRes>>,
    pub uuid: Option<Uuid>,
    pub monitors: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DaemonRes {
    GetVol(f64),
    GetMute(bool),
    GetBri(f64),
    Success,
    Failure(String),
}
