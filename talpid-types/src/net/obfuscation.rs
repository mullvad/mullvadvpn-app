use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use super::{Endpoint, TransportProtocol};

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Debug)]
pub enum ObfuscatorConfig {
    Udp2Tcp { endpoint: SocketAddr },
    Shadowsocks { endpoint: SocketAddr },
}

impl ObfuscatorConfig {
    pub fn get_obfuscator_endpoint(&self) -> Endpoint {
        match self {
            ObfuscatorConfig::Udp2Tcp { endpoint } => Endpoint {
                address: *endpoint,
                protocol: TransportProtocol::Tcp,
            },
            ObfuscatorConfig::Shadowsocks { endpoint } => Endpoint {
                address: *endpoint,
                protocol: TransportProtocol::Udp,
            },
        }
    }
}
