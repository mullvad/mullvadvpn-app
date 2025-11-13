//! App installer

use serde::{Deserialize, Serialize};

use super::architecture::Architecture;

/// App installer
#[derive(Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Installer {
    /// Installer architecture
    pub architecture: Architecture,
    /// Mirrors that host the artifact
    pub urls: Vec<String>,
    /// Size of the installer, in bytes
    pub size: usize,
    /// Hash of the installer, hexadecimal string
    pub sha256: String,
}
