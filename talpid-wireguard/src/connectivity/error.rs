use super::pinger;
use crate::TunnelError;

/// Connectivity monitor errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to read tunnel's configuration
    #[error("Failed to read tunnel's configuration")]
    ConfigReadError(TunnelError),

    /// Failed to send ping
    #[error("Ping failed")]
    PingError(#[from] pinger::Error),
}
