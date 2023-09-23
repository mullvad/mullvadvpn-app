use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Os {
    Linux,
    Macos,
    Windows,
}

impl std::fmt::Display for Os {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Os::Linux => f.write_str("Linux"),
            Os::Macos => f.write_str("macOS"),
            Os::Windows => f.write_str("Windows"),
        }
    }
}

#[cfg(target_os = "linux")]
pub const CURRENT_OS: Os = Os::Linux;

#[cfg(target_os = "windows")]
pub const CURRENT_OS: Os = Os::Windows;

#[cfg(target_os = "macos")]
pub const CURRENT_OS: Os = Os::Macos;
