use std::fmt::{self, Display};

pub enum DaemonErr {
    InitServerFailed,
    ServerAlreadyRunning,
}

pub enum ClientErr {
    CannotConnectServer,
}

impl Display for DaemonErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DaemonErr::InitServerFailed => write!(f, "Daemon error: init server failed"),
            DaemonErr::ServerAlreadyRunning => {
                write!(f, "Daemon error: a server is already running")
            }
        }
    }
}

impl Display for ClientErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientErr::CannotConnectServer => {
                write!(f, "Client error: cannot connect to the server")
            }
        }
    }
}
