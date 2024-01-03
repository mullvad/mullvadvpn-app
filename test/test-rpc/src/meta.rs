use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Os {
    Linux,
    Macos,
    Windows,
}

impl FromStr for Os {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "linux" => Ok(Os::Linux),
            "macos" => Ok(Os::Macos),
            "windows" => Ok(Os::Windows),
            other => Err(format!("unknown os {other}").into()),
        }
    }
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
