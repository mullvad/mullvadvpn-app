use crate::net::{Endpoint, GenericTunnelOptions, TransportProtocol, TunnelEndpoint, TunnelType};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    net::{IpAddr, SocketAddr},
};


#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Debug)]
/// Wireguard tunnel parameters
pub struct TunnelParameters {
    pub connection: ConnectionConfig,
    pub generic_options: GenericTunnelOptions,
    pub options: TunnelOptions,
}

/// Wireguard tunnel options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TunnelOptions {
    /// MTU for the wireguard tunnel
    pub mtu: Option<u16>,
    /// firewall mark
    #[cfg(target_os = "linux")]
    pub fwmark: i32,
}


#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ConnectionConfig {
    pub tunnel: TunnelConfig,
    pub peer: PeerConfig,
    pub gateway: IpAddr,
}

impl ConnectionConfig {
    pub fn get_tunnel_endpoint(&self) -> TunnelEndpoint {
        let host = self.peer.endpoint;
        TunnelEndpoint {
            tunnel_type: TunnelType::Wireguard,
            endpoint: Endpoint {
                address: host,
                protocol: TransportProtocol::Udp,
            },
        }
    }
}

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Debug, Hash)]
pub struct PeerConfig {
    pub public_key: PublicKey,
    pub allowed_ips: Vec<IpNetwork>,
    pub endpoint: SocketAddr,
}

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Debug)]
pub struct TunnelConfig {
    pub private_key: PrivateKey,
    pub addresses: Vec<IpAddr>,
}


impl Default for TunnelOptions {
    fn default() -> TunnelOptions {
        Self {
            mtu: None,
            // Magic value that should be different for different end user applications, used as
            // a firewall marker on Linux.
            #[cfg(target_os = "linux")]
            fwmark: 78_78_78,
        }
    }
}
/// Wireguard x25519 private key
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct PrivateKey([u8; 32]);

impl PrivateKey {
    /// Get private key as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for PrivateKey {
    fn from(private_key: [u8; 32]) -> PrivateKey {
        PrivateKey(private_key)
    }
}

/// Wireguard x25519 public key
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct PublicKey([u8; 32]);

impl PublicKey {
    /// Get the public key as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}


impl From<[u8; 32]> for PublicKey {
    fn from(public_key: [u8; 32]) -> PublicKey {
        PublicKey(public_key)
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> ::std::result::Result<(), fmt::Error> {
        write!(f, "{}", &self)
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> ::std::result::Result<(), fmt::Error> {
        write!(f, "{}", &base64::encode(&self.0))
    }
}
