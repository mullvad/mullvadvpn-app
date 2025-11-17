//! Installer architecture

use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "strum", derive(strum::EnumIter))]
pub enum Architecture {
    /// x86-64 architecture
    X86,
    /// ARM64 architecture
    Arm64,
}

impl Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Architecture::X86 => f.write_str("x86"),
            Architecture::Arm64 => f.write_str("arm64"),
        }
    }
}
