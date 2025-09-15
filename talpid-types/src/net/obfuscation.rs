use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use super::{Endpoint, TransportProtocol};

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Debug)]
pub enum ObfuscatorConfig {
    Udp2Tcp {
        endpoint: SocketAddr,
    },
    Shadowsocks {
        endpoint: SocketAddr,
    },
    Quic {
        hostname: String,
        endpoint: SocketAddr,
        auth_token: String,
    },
    Lwo {
        endpoint: SocketAddr,
    },
    Multiplexer {
        direct: Option<Endpoint>,
        // TODO: prevent recursion
        configs: Vec<ObfuscatorConfig>,
    },
}

impl ObfuscatorConfig {
    pub fn get_obfuscator_endpoint(&self) -> Vec<Endpoint> {
        match self {
            ObfuscatorConfig::Udp2Tcp { endpoint } => vec![Endpoint {
                address: *endpoint,
                protocol: TransportProtocol::Tcp,
            }],
            ObfuscatorConfig::Shadowsocks { endpoint } => vec![Endpoint {
                address: *endpoint,
                protocol: TransportProtocol::Udp,
            }],
            ObfuscatorConfig::Quic { endpoint, .. } => vec![Endpoint {
                address: *endpoint,
                protocol: TransportProtocol::Udp,
            }],
            ObfuscatorConfig::Lwo { endpoint, .. } => vec![Endpoint {
                address: *endpoint,
                protocol: TransportProtocol::Udp,
            }],
            ObfuscatorConfig::Multiplexer { direct, configs } => {
                let mut endpoints = vec![];
                if let Some(direct) = direct {
                    endpoints.push(*direct);
                }
                for config in configs {
                    endpoints.extend(config.get_obfuscator_endpoint());
                }

                endpoints.sort();
                endpoints.dedup();

                endpoints
            }
        }
    }
}
