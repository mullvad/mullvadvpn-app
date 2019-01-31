use crate::{
    endpoint::MullvadEndpoint,
    location::{CityCode, CountryCode, Location},
};

use serde::{Deserialize, Serialize};
use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};
use talpid_types::net::{wireguard, Endpoint, TransportProtocol};


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

impl OpenVpnEndpointData {
    pub fn into_mullvad_endpoint(self, host: IpAddr) -> MullvadEndpoint {
        MullvadEndpoint::OpenVpn(Endpoint::new(host, self.port, self.protocol))
    }
}

impl fmt::Display for OpenVpnEndpointData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{} port {}", self.protocol, self.port)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Deserialize, Serialize, Debug)]
pub struct WireguardEndpointData {
    /// Port to connect to
    pub port_ranges: Vec<(u16, u16)>,
    /// Gateways to be used with the tunnel
    pub ipv4_gateway: Ipv4Addr,
    pub ipv6_gateway: Ipv6Addr,
    /// The peer's public key
    pub public_key: wireguard::PublicKey,
}

impl fmt::Display for WireguardEndpointData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "gateways {} - {} port_ranges {{ {} }} public_key {}",
            self.ipv4_gateway,
            self.ipv6_gateway,
            self.port_ranges
                .iter()
                .map(|range| format!("[{} - {}]", range.0, range.1))
                .collect::<Vec<_>>()
                .join(","),
            self.public_key,
        )
    }
}
