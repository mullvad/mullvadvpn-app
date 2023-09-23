use serde::{Deserialize, Serialize};

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub const SOCKET_PATH: &str = "/var/run/mullvad-vpn";
#[cfg(windows)]
pub const SOCKET_PATH: &str = "//./pipe/Mullvad VPN";

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Error {
    ConnectError,
    DisconnectError,
    DaemonError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum ServiceStatus {
    NotRunning,
    Running,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Verbosity {
    Info,
    Debug,
    Trace,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MullvadClientVersion {
    None,
    New,
    Previous,
}
