use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum DaemonEvt {
    CloseWindow,
    ShutDown,
    GetVol,
    SetVol(u32),
    IncVol(u32),
    DecVol(u32),
}
