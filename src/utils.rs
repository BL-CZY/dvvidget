use crate::daemon::structs::DaemonCmd;

#[derive(Debug)]
pub enum DaemonErr {
    InitServerFailed,
    ServerAlreadyRunning,
    ReadingFailed(String),
    DeserializeError(String),
    SendFailed(DaemonCmd),
    ShutdownFailed(String),
}

#[derive(Debug)]
pub enum ClientErr {
    CannotConnectServer,
}
