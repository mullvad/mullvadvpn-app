use std::path::PathBuf;

#[cfg(target_os = "android")]
use serde::{Deserialize, Serialize};

/// Whether split tunneling operates in exclusion or inclusion mode (Android only).
#[cfg(target_os = "android")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitTunnelMode {
    /// Listed apps bypass the VPN tunnel.
    #[default]
    Exclude,
    /// Only listed apps use the VPN tunnel; all other traffic bypasses it.
    Include,
}

/// A process that is being excluded from the tunnel.
#[derive(Debug, Clone)]
pub struct ExcludedProcess {
    /// Process identifier.
    pub pid: u32,
    /// Path to the image that this process is an instance of.
    pub image: PathBuf,
    /// If true, then the process is split because its parent was split,
    /// not due to its path being in the config.
    pub inherited: bool,
}
