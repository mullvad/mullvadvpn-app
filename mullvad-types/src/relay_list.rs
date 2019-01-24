use crate::location::{CityCode, CountryCode, Location};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    net::{IpAddr, Ipv4Addr},
};
use talpid_types::net::{openvpn, wireguard, Endpoint, TransportProtocol};


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelayList {
    pub countries: Vec<RelayListCountry>,
}

impl RelayList {
    pub fn empty() -> Self {
        Self {
            countries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelayListCountry {
    pub name: String,
    pub code: CountryCode,
    pub cities: Vec<RelayListCity>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelayListCity {
    pub name: String,
    pub code: CityCode,
    pub latitude: f64,
    pub longitude: f64,
    pub relays: Vec<Relay>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Relay {
    pub hostname: String,
    pub ipv4_addr_in: Ipv4Addr,
    pub include_in_country: bool,
    pub weight: u64,
    #[serde(skip_serializing_if = "RelayTunnels::is_empty", default)]
    pub tunnels: RelayTunnels,
    #[serde(skip)]
    pub location: Option<Location>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct RelayTunnels {
    pub openvpn: Vec<OpenVpnEndpointData>,
    #[serde(skip)]
    pub wireguard: Vec<WireguardEndpointData>,
}

impl RelayTunnels {
    pub fn is_empty(&self) -> bool {
        self.openvpn.is_empty() && self.wireguard.is_empty()
    }

    pub fn clear(&mut self) {
        self.openvpn.clear();
        self.wireguard.clear();
    }
}

/// Represents one tunnel endpoint. Address, plus extra parameters specific to tunnel protocol.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct TunnelEndpoint {
    pub address: IpAddr,
    pub tunnel: TunnelEndpointData,
}

impl TunnelEndpoint {
    /// Returns this tunnel endpoint as an `Endpoint`.
    pub fn to_endpoint(&self) -> Endpoint {
        Endpoint::new(
            self.address,
            self.tunnel.port(),
            self.tunnel.transport_protocol(),
        )
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct OpenVpnEndpointData {
    pub port: u16,
    pub protocol: TransportProtocol,
}

impl fmt::Display for OpenVpnEndpointData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{} port {}", self.protocol, self.port)
    }
}

impl From<&openvpn::ConnectionConfig> for OpenVpnEndpointData {
    fn from(config: &openvpn::ConnectionConfig) -> OpenVpnEndpointData {
        OpenVpnEndpointData {
            port: config.host.port(),
            protocol: config.protocol,
        }
    }
}


#[derive(Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct WireguardEndpointData {
    /// Port to connect to
    pub port: u16,
    /// Peer's IP address
    pub gateway: IpAddr,
    /// The peer's public key
    pub peer_public_key: wireguard::PublicKey,
}

impl fmt::Debug for WireguardEndpointData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct(&"WireguardEndpointData")
            .field("port", &self.port)
            .field("gateway", &self.gateway)
            .field("peer_public_key", &self.peer_public_key)
            .finish()
    }
}

impl fmt::Display for WireguardEndpointData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "gateway {} port {} peer_public_key {}",
            self.gateway, self.port, self.peer_public_key,
        )
    }
}

impl From<&wireguard::ConnectionConfig> for WireguardEndpointData {
    fn from(config: &wireguard::ConnectionConfig) -> WireguardEndpointData {
        WireguardEndpointData {
            port: config.peer.endpoint.port(),
            gateway: config.gateway,
            peer_public_key: config.peer.public_key.clone(),
        }
    }
}
