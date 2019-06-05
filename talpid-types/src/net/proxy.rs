use crate::net::Endpoint;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Types of bridges that can be used to proxy a connection to a tunnel
#[serde(rename_all = "snake_case")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProxyType {
    /// Shadowsocks
    Shadowsocks,
    /// Custom bridge
    Custom,
}

impl fmt::Display for ProxyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let bridge = match self {
            ProxyType::Shadowsocks => "Shadowsocks",
            ProxyType::Custom => "custom bridge",
        };
        write!(f, "{}", bridge)
    }
}


/// Bridge endpoint, broadcast as part of TunnelEndpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProxyEndpoint {
    #[serde(flatten)]
    pub endpoint: Endpoint,
    pub proxy_type: ProxyType,
}
