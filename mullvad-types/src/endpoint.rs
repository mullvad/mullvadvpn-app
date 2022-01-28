use serde::{Deserialize, Serialize};
use std::{
    fmt,
    net::{Ipv4Addr, Ipv6Addr},
};
use talpid_types::net::{wireguard, Endpoint, TransportProtocol};

use crate::relay_list::{OpenVpnEndpointData, WireguardEndpointData};

/// Contains server data needed to connect to a single mullvad endpoint
#[derive(Debug, Clone)]
pub enum MullvadEndpoint {
    OpenVpn(Endpoint),
    Wireguard(MullvadWireguardEndpoint),
}

/// Contains WireGuard server data needed to connect to a WireGuard endpoint
#[derive(Debug, Clone)]
pub struct MullvadWireguardEndpoint {
    pub peer: wireguard::PeerConfig,
    pub exit_peer: Option<wireguard::PeerConfig>,
    pub ipv4_gateway: Ipv4Addr,
    pub ipv6_gateway: Ipv6Addr,
}

impl MullvadEndpoint {
    /// Returns this tunnel endpoint as an `Endpoint`.
    pub fn to_endpoint(&self) -> Endpoint {
        match self {
            MullvadEndpoint::OpenVpn(endpoint) => *endpoint,
            MullvadEndpoint::Wireguard(wireguard_relay) => Endpoint::new(
                wireguard_relay.peer.endpoint.ip(),
                wireguard_relay.peer.endpoint.port(),
                TransportProtocol::Udp,
            ),
        }
    }

    pub fn unwrap_wireguard(&self) -> &MullvadWireguardEndpoint {
        match self {
            Self::Wireguard(endpoint) => endpoint,
            other => {
                panic!("Expected WireGuard enum variant but got {:?}", other);
            }
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
