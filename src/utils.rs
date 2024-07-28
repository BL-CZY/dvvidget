use crate::daemon::structs::DaemonEvt;

#[derive(Debug)]
pub enum DaemonErr {
    InitServerFailed,
    ServerAlreadyRunning,
    ReadingFailed(String),
    DeserializeError(String),
    SendFailed(DaemonEvt),
    ShutdownFailed(String),
}

#[derive(Debug)]
pub enum ClientErr {
    CannotConnectServer,
}
