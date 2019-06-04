use crate::net::Endpoint;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Bridge types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BridgeType {
    /// Shadowsocks
    #[serde(rename = "shadowsocks")]
    Shadowsocks,
    /// Custom bridge
    #[serde(rename = "custom")]
    Custom,
}

impl fmt::Display for BridgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let bridge = match self {
            BridgeType::Shadowsocks => "Shadowsocks",
            BridgeType::Custom => "custom bridge",
        };
        write!(f, "{}", bridge)
    }
}


/// Bridge endpoint, broadcast as part of TunnelEndpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BridgeEndpoint {
    #[serde(flatten)]
    pub bridge_endpoint: Endpoint,
    pub bridge_type: BridgeType,
}
