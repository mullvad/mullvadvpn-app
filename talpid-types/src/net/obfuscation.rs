use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use vec1::Vec1;

use super::{Endpoint, TransportProtocol};

/// Available obfuscation configuration types.
#[derive(Clone, Eq, PartialEq, Debug)]
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
        configs: Vec1<ObfuscatorConfig>,
    },
}

#[derive(Clone, Eq, PartialEq, Hash, Deserialize, Serialize, Debug)]
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
        version: LwoVersion,
    },
}

/// The version of the LWO protocol.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum LwoVersion {
    V1,
    V2,
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
        Some(Obfuscators::Multiplexer {
            direct,
            configs: Vec1::try_from(obfuscators).ok()?,
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
            Obfuscators::Multiplexer { direct, configs } => {
                let mut endpoints = vec![];
                if let Some(direct) = direct {
                    endpoints.push(Endpoint {
                        address: *direct,
                        protocol: TransportProtocol::Udp,
                    });
                }
                endpoints.extend(configs.iter().map(|cfg| cfg.endpoint()));

                endpoints.sort();
                endpoints.dedup();

                endpoints
            }
        }
    }

    pub fn obfuscator_configs(&self) -> Vec<ObfuscatorConfig> {
        match self {
            Obfuscators::Single(config) => vec![config.clone()],
            Obfuscators::Multiplexer { configs, .. } => configs.iter().cloned().collect(),
        }
    }

    /// Return the socket address of the direct/plain wireguard connection inside
    /// the multiplexer, if present.
    pub fn direct(&self) -> Option<SocketAddr> {
        match self {
            Obfuscators::Single(_) => None,
            Obfuscators::Multiplexer { direct, .. } => *direct,
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
