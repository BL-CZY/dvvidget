use std::fmt::{self, Display};

pub enum DaemonErr {
    InitServerFailed,
    ServerAlreadyRunning,
}

impl Display for DaemonErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DaemonErr::InitServerFailed => write!(f, "init server failed"),
            DaemonErr::ServerAlreadyRunning => write!(f, "a server is already running"),
        }
    }
}
