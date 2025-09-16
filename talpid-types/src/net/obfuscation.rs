use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use super::{Endpoint, TransportProtocol};

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Debug)]
pub enum Obfuscators {
    Single(ObfuscatorConfig),
    Multiplexer {
        // TODO: Replace with socket address
        direct: Option<Endpoint>,
        configs: (ObfuscatorConfig, Vec<ObfuscatorConfig>),
    },
}

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
}

impl Obfuscators {
    /// Return a [Obfuscators::Multiplexer]. If `obfuscators` contains zero values,
    /// this returns `None`.
    pub fn multiplexer(direct: Option<Endpoint>, obfuscators: &[ObfuscatorConfig]) -> Option<Self> {
        let [first, remaining @ ..] = &obfuscators[..] else {
            return None;
        };
        Some(Obfuscators::Multiplexer {
            direct,
            configs: (first.clone(), remaining.to_vec()),
        })
    }

    /// Return all potential obfuscation endpoints
    pub fn endpoints(&self) -> Vec<Endpoint> {
        match self {
            Obfuscators::Single(config) => vec![config.endpoint()],
            Obfuscators::Multiplexer {
                direct,
                configs: (first_config, remaining_configs),
            } => {
                let mut endpoints = vec![];
                if let Some(direct) = direct {
                    endpoints.push(*direct);
                }
                endpoints.push(first_config.endpoint());
                endpoints.extend(remaining_configs.iter().map(|cfg| cfg.endpoint()));

                endpoints.sort();
                endpoints.dedup();

                endpoints
            }
        }
    }
}

impl ObfuscatorConfig {
    /// Return obfuscation endpoint
    pub fn endpoint(&self) -> Endpoint {
        match self {
            ObfuscatorConfig::Udp2Tcp { endpoint } => Endpoint {
                address: *endpoint,
                protocol: TransportProtocol::Tcp,
            },
            ObfuscatorConfig::Shadowsocks { endpoint } => Endpoint {
                address: *endpoint,
                protocol: TransportProtocol::Udp,
            },
            ObfuscatorConfig::Quic { endpoint, .. } => Endpoint {
                address: *endpoint,
                protocol: TransportProtocol::Udp,
            },
            ObfuscatorConfig::Lwo { endpoint, .. } => Endpoint {
                address: *endpoint,
                protocol: TransportProtocol::Udp,
            },
        }
    }
}
