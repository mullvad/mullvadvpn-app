use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, ops::Deref};
use vec1::Vec1;

use super::{Endpoint, TransportProtocol};

/// Available obfuscation configuration types.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Obfuscators(pub Vec1<TransportConfig>);

impl Deref for Obfuscators {
    type Target = Vec1<TransportConfig>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TransportConfig {
    Direct(SocketAddr),
    Obfuscation(ObfuscatorConfig),
}

impl TransportConfig {
    pub fn endpoint(&self) -> Endpoint {
        match self {
            TransportConfig::Direct(address) => Endpoint {
                address: *address,
                protocol: TransportProtocol::Udp,
            },
            TransportConfig::Obfuscation(obfuscator_config) => obfuscator_config.endpoint(),
        }
    }
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
    /// A single direct WireGuard connection, without any obfuscation.
    pub fn direct(endpoint: SocketAddr) -> Self {
        Obfuscators(vec1::vec1![TransportConfig::Direct(endpoint)])
    }

    /// Return all potential endpoints that this obfuscation configuration might connect to.
    ///
    /// Returns all possible endpoints (direct + all obfuscated methods) that the multiplexer
    /// might use, with duplicates removed.
    pub fn endpoints(&self) -> Vec<Endpoint> {
        self.0
            .iter()
            .map(|t| t.endpoint())
            .unique()
            .sorted()
            .collect()
    }

    // /// Return the socket address of the direct/plain wireguard connection inside
    // /// the multiplexer, if present.
    // pub fn direct(&self) -> Option<SocketAddr> {
    //     match self {
    //         Obfuscators::Single(_) => None,
    //         Obfuscators::Multiplexer { direct, .. } => *direct,
    //     }
    // }

    pub fn obfuscator_configs(&self) -> Vec<ObfuscatorConfig> {
        self.iter()
            .filter_map(|t| match t {
                TransportConfig::Direct(_) => None,
                TransportConfig::Obfuscation(obfuscator_config) => Some(obfuscator_config),
            })
            .cloned()
            .collect()
    }

    /// Whether this obfuscation configuration is single a direct connection.
    pub fn is_direct(&self) -> bool {
        self.0.len() == 1 && matches!(self.0.first(), TransportConfig::Direct(_))
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
