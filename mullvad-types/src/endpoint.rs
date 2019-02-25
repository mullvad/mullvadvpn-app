use serde::{Deserialize, Serialize};
use std::{
    fmt,
    net::{Ipv4Addr, Ipv6Addr},
};
use talpid_types::net::{wireguard, Endpoint, TransportProtocol};

use crate::relay_list::{OpenVpnEndpointData, WireguardEndpointData};


/// Contains server data needed to conenct to a single mullvad endpoint
#[derive(Debug, Clone)]
pub enum MullvadEndpoint {
    OpenVpn(Endpoint),
    Wireguard {
        peer: wireguard::PeerConfig,
        ipv4_gateway: Ipv4Addr,
        ipv6_gateway: Ipv6Addr,
    },
}

impl MullvadEndpoint {
    /// Returns this tunnel endpoint as an `Endpoint`.
    pub fn to_endpoint(&self) -> Endpoint {
        match self {
            MullvadEndpoint::OpenVpn(endpoint) => *endpoint,
            MullvadEndpoint::Wireguard {
                peer,
                ipv4_gateway: _,
                ipv6_gateway: _,
            } => Endpoint::new(
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
