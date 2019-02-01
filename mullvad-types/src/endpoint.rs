use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    net::{IpAddr, SocketAddr},
};
use talpid_types::net::{wireguard, Endpoint, TransportProtocol};

use crate::relay_list::{OpenVpnEndpointData, WireguardEndpointData};

/// Contains server data needed to conenct to a single mullvad endpoint
#[derive(Debug, Clone)]
pub enum MullvadEndpoint {
    OpenVpn(Endpoint),
    Wireguard {
        peer: wireguard::PeerConfig,
        gateway: IpAddr,
    },
}

impl MullvadEndpoint {
    /// Returns this tunnel endpoint as an `Endpoint`.
    pub fn to_endpoint(&self) -> Endpoint {
        match self {
            MullvadEndpoint::OpenVpn(endpoint) => *endpoint,
            MullvadEndpoint::Wireguard { peer, gateway: _ } => Endpoint::new(
                peer.endpoint.ip(),
                peer.endpoint.port(),
                TransportProtocol::Udp,
            ),
        }
    }
}
/// TunnelEndpointData contains data required to connect to a given tunnel endpoint.
/// Different endpoint types can require different types of data.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum TunnelEndpointData {
    /// Extra parameters for an OpenVPN tunnel endpoint.
    #[serde(rename = "openvpn")]
    OpenVpn(OpenVpnEndpointData),
    /// Extra parameters for a Wireguard tunnel endpoint.
    #[serde(rename = "wireguard")]
    Wireguard(WireguardEndpointData),
}
impl From<OpenVpnEndpointData> for TunnelEndpointData {
    fn from(endpoint_data: OpenVpnEndpointData) -> TunnelEndpointData {
        TunnelEndpointData::OpenVpn(endpoint_data)
    }
}

impl From<WireguardEndpointData> for TunnelEndpointData {
    fn from(endpoint_data: WireguardEndpointData) -> TunnelEndpointData {
        TunnelEndpointData::Wireguard(endpoint_data)
    }
}

impl fmt::Display for TunnelEndpointData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            TunnelEndpointData::OpenVpn(openvpn_data) => {
                write!(f, "OpenVPN ")?;
                openvpn_data.fmt(f)
            }
            TunnelEndpointData::Wireguard(wireguard_data) => {
                write!(f, "Wireguard ")?;
                wireguard_data.fmt(f)
            }
        }
    }
}

impl TunnelEndpointData {
    pub fn to_mullvad_endpoint(self, host: IpAddr) -> MullvadEndpoint {
        match self {
            TunnelEndpointData::OpenVpn(metadata) => {
                MullvadEndpoint::OpenVpn(Endpoint::new(host, metadata.port, metadata.protocol))
            }
            TunnelEndpointData::Wireguard(metadata) => {
                let peer_config = wireguard::PeerConfig {
                    public_key: metadata.peer_public_key,
                    endpoint: SocketAddr::new(host, metadata.port),
                    allowed_ips: all_of_the_internet(),
                };
                MullvadEndpoint::Wireguard {
                    peer: peer_config,
                    gateway: metadata.gateway,
                }
            }
        }
    }
    pub fn port(&self) -> u16 {
        match self {
            TunnelEndpointData::OpenVpn(metadata) => metadata.port,
            TunnelEndpointData::Wireguard(metadata) => metadata.port,
        }
    }

    pub fn transport_protocol(&self) -> TransportProtocol {
        match self {
            TunnelEndpointData::OpenVpn(metadata) => metadata.protocol,
            TunnelEndpointData::Wireguard(_) => TransportProtocol::Udp,
        }
    }
}

pub fn all_of_the_internet() -> Vec<IpNetwork> {
    vec![
        "0.0.0.0/0".parse().expect("Failed to parse ipv6 network"),
        "::0/0".parse().expect("Failed to parse ipv6 network"),
    ]
}
