use crate::location::{CityCode, CountryCode, Location};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    net::{IpAddr, Ipv4Addr},
};
use talpid_types::net::{wireguard, TransportProtocol};


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
