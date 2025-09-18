use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use super::{Endpoint, TransportProtocol};

/// Available obfuscation configuration types.
#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Debug)]
pub enum Obfuscators {
    /// A single obfuscation method
    Single(ObfuscatorConfig),
    /// Try multiple obfuscation methods (using `multiplexer` obfuscation).
    ///
    /// They are tested in the following order: `direct`, `config.0`, then
    /// the remaining configs in `configs.1` in order.
    Multiplexer {
        /// Optional direct connection (no obfuscation) to try along with `configs`.
        direct: Option<SocketAddr>,
        /// Obfuscation configurations to try.
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
    /// Create a multiplexer obfuscator configuration.
    ///
    /// See [Obfuscators::Multiplexer] for more details.
    ///
    /// # Arguments
    /// * `direct` - Optional direct connection endpoint (no obfuscation)
    /// * `obfuscators` - List of obfuscation methods to try, at least one.
    ///
    /// # Returns
    /// * `Some(Obfuscators::Multiplexer)` if at least one obfuscation method is provided
    /// * `None` if the obfuscators list is empty
    pub fn multiplexer(
        direct: Option<SocketAddr>,
        obfuscators: &[ObfuscatorConfig],
    ) -> Option<Self> {
        let [first, remaining @ ..] = obfuscators else {
            return None;
        };
        Some(Obfuscators::Multiplexer {
            direct,
            configs: (first.clone(), remaining.to_vec()),
        })
    }

    /// Return all potential endpoints that this obfuscation configuration might connect to.
    ///
    /// For single obfuscators, return one endpoint. For `Obfuscators::Multiplexer`, return
    /// all possible endpoints (direct + all obfuscated methods) that the multiplexer
    /// might use, with duplicates removed.
    pub fn endpoints(&self) -> Vec<Endpoint> {
        match self {
            Obfuscators::Single(config) => vec![config.endpoint()],
            Obfuscators::Multiplexer {
                direct,
                configs: (first_config, remaining_configs),
            } => {
                let mut endpoints = vec![];
                if let Some(direct) = direct {
                    endpoints.push(Endpoint {
                        address: *direct,
                        protocol: TransportProtocol::Udp,
                    });
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
    /// Return obfuscation endpoint, i.e. the first remote hop that will be connected to
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
